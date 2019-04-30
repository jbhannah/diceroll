use rand::distributions::{Distribution, Uniform};
use regex::Regex;
use std::error::Error;
use std::fmt;

#[derive(Debug)]
pub struct DiceExprError {
    expr: String,
}

impl Error for DiceExprError {}

impl fmt::Display for DiceExprError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Invalid dice expression \"{}\"", self.expr)
    }
}

#[derive(PartialEq, Debug)]
enum Drop {
    DropHigh,
    DropLow,
}

#[derive(PartialEq, Debug)]
pub struct Dice {
    count: u16,
    sides: u16,
    modifier: i16,
    drop: Option<Drop>,
}

impl Dice {
    pub fn new(expr: &str) -> Result<Dice, DiceExprError> {
        lazy_static! {
            static ref RE: Regex = Regex::new(r"^(\d+)?d(\d+)([+-]\d+)?(-[LlHh])?$").unwrap();
        }

        let caps;
        if let Some(c) = RE.captures(&expr) {
            caps = c;
        } else {
            return Err(DiceExprError {
                expr: expr.to_string(),
            });
        }

        let mut count: u16 = 1;
        if let Some(c) = caps.get(1) {
            count = c.as_str().parse().unwrap();
        }

        let sides: u16;
        if let Some(c) = caps.get(2) {
            sides = c.as_str().parse().unwrap();
        } else {
            return Err(DiceExprError {
                expr: expr.to_string(),
            });
        }

        let mut modifier: i16 = 0;
        if let Some(c) = caps.get(3) {
            modifier = match c.as_str().parse::<i16>() {
                Ok(n) if -n < (count * sides) as i16 => n,
                _ => {
                    return Err(DiceExprError {
                        expr: expr.to_string(),
                    })
                }
            };
        };

        let mut drop: Option<Drop> = None;
        if let Some(c) = caps.get(4) {
            if count > 1 {
                drop = Some(match c.as_str().to_lowercase().as_str() {
                    "-h" => Drop::DropHigh,
                    "-l" => Drop::DropLow,
                    _ => {
                        return Err(DiceExprError {
                            expr: expr.to_string(),
                        })
                    }
                })
            } else {
                return Err(DiceExprError {
                    expr: expr.to_string(),
                });
            }
        }

        Ok(Dice {
            count: count,
            sides: sides,
            modifier: modifier,
            drop: drop,
        })
    }

    pub fn expr(&self) -> String {
        format!(
            "{}d{}{}{}",
            self.count,
            self.sides,
            match self.modifier {
                n if n > 0 => format!("+{}", n),
                n if n < 0 => format!("{}", n),
                _ => String::from(""),
            },
            match self.drop {
                Some(Drop::DropHigh) => "-H",
                Some(Drop::DropLow) => "-L",
                None => "",
            }
        )
    }

    pub fn roll(&self) -> u16 {
        let mut rng = ::rand::thread_rng();
        let roll = self.sample(&mut rng).into_iter();

        let sum: u16 = roll.clone().sum::<u16>()
            - match self.drop {
                Some(Drop::DropHigh) => roll.max().unwrap(),
                Some(Drop::DropLow) => roll.min().unwrap(),
                None => 0,
            };

        if -self.modifier < sum as i16 {
            (sum as i16 + self.modifier) as u16
        } else {
            0
        }
    }

    fn sample<R: ::rand::Rng>(&self, rng: &mut R) -> Vec<u16> {
        Uniform::from(1..self.sides + 1)
            .sample_iter(rng)
            .take(self.count as usize)
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rand::rngs::mock::StepRng;

    #[test]
    fn parse_valid() {
        let dice = Dice::new("4d4").unwrap();
        assert_eq!(
            Dice {
                count: 4,
                sides: 4,
                modifier: 0,
                drop: None,
            },
            dice
        );
    }

    #[test]
    fn parse_modifier() {
        let dice = Dice::new("4d4+4-H").unwrap();
        assert_eq!(
            Dice {
                count: 4,
                sides: 4,
                modifier: 4,
                drop: Some(Drop::DropHigh),
            },
            dice
        );
    }

    #[test]
    fn parse_invalid() {
        assert!(
            Dice::new("asdf").is_err(),
            "Invalid dice expression \"asdf\""
        );
    }

    #[test]
    fn expr() {
        let dice = Dice::new("4d4-4-l").unwrap();
        assert_eq!("4d4-4-L", dice.expr());
    }

    #[test]
    fn roll() {
        let dice = Dice::new("4d4").unwrap();
        let r = dice.roll();
        assert!(r >= 4);
        assert!(r <= 16);
    }

    #[test]
    fn sample() {
        let dice = Dice::new("4d4").unwrap();
        let mut rng = StepRng::new(1, 0);
        let s = dice.sample(&mut rng);
        assert_eq!(s, vec![1, 1, 1, 1]);
    }
}
