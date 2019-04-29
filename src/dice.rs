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
pub struct Dice {
    count: u16,
    sides: u16,
    modifier: i16,
}

impl Dice {
    pub fn new(expr: &str) -> Result<Dice, DiceExprError> {
        lazy_static! {
            static ref RE: Regex = Regex::new(r"^(\d+)?d(\d+)([+-]\d+)?$").unwrap();
        }

        let caps = match RE.captures(&expr) {
            Some(c) => c,
            None => {
                return Err(DiceExprError {
                    expr: expr.to_string(),
                })
            }
        };

        let count = match caps.get(1) {
            Some(c) => c.as_str().parse::<u16>().unwrap(),
            None => 1,
        };

        let sides = match caps.get(2) {
            Some(c) => c.as_str().parse::<u16>().unwrap(),
            None => {
                return Err(DiceExprError {
                    expr: expr.to_string(),
                })
            }
        };

        let modifier = match caps.get(3) {
            Some(c) => match c.as_str().parse::<i16>() {
                Ok(n) if -n < (count * sides) as i16 => n,
                _ => {
                    return Err(DiceExprError {
                        expr: expr.to_string(),
                    })
                }
            },
            None => 0,
        };

        Ok(Dice {
            count: count,
            sides: sides,
            modifier: modifier,
        })
    }

    pub fn expr(&self) -> String {
        format!(
            "{}d{}{}",
            self.count,
            self.sides,
            match self.modifier {
                n if n > 0 => format!("+{}", n),
                n if n < 0 => format!("{}", n),
                _ => String::from(""),
            }
        )
    }

    pub fn roll(&self) -> u16 {
        let mut rng = ::rand::thread_rng();
        let roll = self.sample(&mut rng).into_iter();
        (roll.sum::<u16>() as i16 + self.modifier) as u16
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
                modifier: 0
            },
            dice
        );
    }

    #[test]
    fn parse_modifier() {
        let dice = Dice::new("4d4+4").unwrap();
        assert_eq!(
            Dice {
                count: 4,
                sides: 4,
                modifier: 4
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
        let dice = Dice::new("4d4-4").unwrap();
        assert_eq!("4d4-4", dice.expr());
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
