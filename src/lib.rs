#[macro_use]
extern crate lazy_static;
extern crate rand;
extern crate regex;

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
    None,
}

impl fmt::Display for Drop {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "{}",
            match self {
                Drop::DropHigh => "-H",
                Drop::DropLow => "-L",
                Drop::None => "",
            }
        )
    }
}

#[derive(PartialEq, Debug)]
pub struct Dice {
    count: u16,
    sides: u16,
    modifier: i16,
    drop: Drop,
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

        let mut drop = Drop::None;
        if let Some(c) = caps.get(4) {
            if count > 1 {
                drop = match c.as_str().to_lowercase().as_str() {
                    "-h" => Drop::DropHigh,
                    "-l" => Drop::DropLow,
                    _ => {
                        return Err(DiceExprError {
                            expr: expr.to_string(),
                        })
                    }
                }
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
            self.drop,
        )
    }

    pub fn roll(&self) -> (u16, Vec<u16>) {
        let mut rng = ::rand::thread_rng();
        let rolls = self.sample(&mut rng);
        let roll = rolls.clone().into_iter();

        let sum: u16 = roll.clone().sum::<u16>()
            - match self.drop {
                Drop::DropHigh => roll.max().unwrap(),
                Drop::DropLow => roll.min().unwrap(),
                Drop::None => 0,
            };

        (
            if -self.modifier < sum as i16 {
                (sum as i16 + self.modifier) as u16
            } else {
                0
            },
            rolls,
        )
    }

    fn sample<R: ::rand::Rng>(&self, rng: &mut R) -> Vec<u16> {
        Uniform::from(1..self.sides + 1)
            .sample_iter(rng)
            .take(self.count as usize)
            .collect()
    }
}

impl fmt::Display for Dice {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.expr())
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
                drop: Drop::None,
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
                drop: Drop::DropHigh,
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
        let (r, _) = dice.roll();
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
