use cfg_if::cfg_if;
use lazy_static::lazy_static;
use rand::thread_rng;
use regex::Regex;
use std::convert::TryFrom;
use std::error::Error;
use std::fmt;
use std::num::ParseIntError;

cfg_if! {
    if #[cfg(test)] {
        use crate::die::MockDie as Die;
    } else {
        use crate::die::Die;
    }
}

#[derive(Debug, PartialEq)]
pub enum DiceExprError {
    Expr(String),
    ParseIntError(ParseIntError),
    Drop(String),
}

impl Error for DiceExprError {}

impl From<ParseIntError> for DiceExprError {
    fn from(e: ParseIntError) -> Self {
        Self::ParseIntError(e)
    }
}

impl From<String> for DiceExprError {
    fn from(s: String) -> Self {
        Self::Expr(s)
    }
}

impl fmt::Display for DiceExprError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Self::Expr(s) => write!(f, "Invalid dice expression \"{}\"", s),
            Self::ParseIntError(e) => write!(f, "Integer parsing error: {}", e),
            Self::Drop(s) => write!(f, "Invalid drop modifier \"{}\"", s),
        }
    }
}

#[derive(Debug, PartialEq)]
enum Drop {
    High,
    Low,
    None,
}

impl TryFrom<&str> for Drop {
    type Error = DiceExprError;

    fn try_from(s: &str) -> Result<Self, Self::Error> {
        match s.to_lowercase().as_str() {
            "h" => Ok(Drop::High),
            "l" => Ok(Drop::Low),
            _ => Err(Self::Error::Drop(s.to_string())),
        }
    }
}

impl fmt::Display for Drop {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "{}",
            match self {
                Drop::High => "-H",
                Drop::Low => "-L",
                Drop::None => "",
            }
        )
    }
}

#[derive(Debug, PartialEq)]
pub struct DiceExpr {
    count: u16,
    sides: u16,
    modifier: i16,
    drop: Drop,
}

impl TryFrom<&str> for DiceExpr {
    type Error = DiceExprError;

    fn try_from(s: &str) -> Result<Self, Self::Error> {
        lazy_static! {
            static ref RE: Regex = Regex::new(r"^(\d+)?d(\d+)([+-]\d+)?(?:-([LlHh]))?$").unwrap();
        }

        let expr = s.to_string();

        if let Some(caps) = RE.captures(s) {
            let count: u16 = match caps.get(1) {
                Some(c) => c.as_str().parse()?,
                None => 1,
            };

            let sides: u16 = match caps.get(2) {
                Some(c) => c.as_str().parse()?,
                None => return Err(Self::Error::from(expr)),
            };

            let modifier: i16 = match caps.get(3) {
                Some(c) => match c.as_str().parse::<i16>() {
                    Ok(n) if -n < (count * sides) as i16 => n,
                    Ok(_) => return Err(Self::Error::from(expr)),
                    Err(e) => return Err(Self::Error::from(e)),
                },
                None => 0,
            };

            let drop = match caps.get(4) {
                Some(s) => match count {
                    1 => return Err(Self::Error::from(expr)),
                    _ => Drop::try_from(s.as_str())?,
                },
                None => Drop::None,
            };

            Ok(DiceExpr {
                count,
                sides,
                modifier,
                drop,
            })
        } else {
            Err(Self::Error::from(expr))
        }
    }
}

impl fmt::Display for DiceExpr {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "{}d{}{}{}",
            match self.count {
                1 => String::from(""),
                n => format!("{}", n),
            },
            self.sides,
            match self.modifier {
                n if n > 0 => format!("+{}", n),
                n if n < 0 => format!("{}", n),
                _ => String::from(""),
            },
            self.drop
        )
    }
}

impl DiceExpr {
    pub fn roll(&self) -> (u16, Vec<u16>) {
        let mut rng = thread_rng();
        let rolls: Vec<u16> = (0..self.count)
            .map(|_| Die::new(self.sides).roll(&mut rng))
            .collect();

        let sum: u16 = rolls.iter().sum::<u16>()
            - match self.drop {
                Drop::High => rolls.iter().max().unwrap(),
                Drop::Low => rolls.iter().min().unwrap(),
                Drop::None => &0,
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
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn try_from_str() {
        let expr = "4d4";

        assert_eq!(
            Ok(DiceExpr {
                count: 4,
                sides: 4,
                modifier: 0,
                drop: Drop::None,
            }),
            DiceExpr::try_from(expr)
        )
    }

    #[test]
    fn try_from_str_modifier() {
        let expr = "4d4+1";

        assert_eq!(
            Ok(DiceExpr {
                count: 4,
                sides: 4,
                modifier: 1,
                drop: Drop::None,
            }),
            DiceExpr::try_from(expr)
        )
    }

    #[test]
    fn try_from_str_modifier_lt_0() {
        let expr = "4d4-1";

        assert_eq!(
            Ok(DiceExpr {
                count: 4,
                sides: 4,
                modifier: -1,
                drop: Drop::None,
            }),
            DiceExpr::try_from(expr)
        )
    }

    #[test]
    fn try_from_str_modifier_too_negative() {
        let expr = "4d4-16";

        assert_eq!(
            Err(DiceExprError::Expr(String::from(expr))),
            DiceExpr::try_from(expr)
        )
    }

    #[test]
    fn try_from_drop_high() {
        let expr = "4d4-H";

        assert_eq!(
            Ok(DiceExpr {
                count: 4,
                sides: 4,
                modifier: 0,
                drop: Drop::High,
            }),
            DiceExpr::try_from(expr)
        )
    }

    #[test]
    fn try_from_drop_single_die() {
        let expr = "d4-H";

        assert_eq!(
            Err(DiceExprError::Expr(String::from(expr))),
            DiceExpr::try_from(expr)
        )
    }

    #[test]
    fn try_from_drop_invalid() {
        let expr = "4d4-J";

        assert_eq!(
            Err(DiceExprError::Drop(String::from("j"))),
            DiceExpr::try_from(expr)
        )
    }

    #[test]
    fn try_from_str_invalid() {
        let expr = "asdf";

        assert_eq!(
            Err(DiceExprError::Expr(String::from(expr))),
            DiceExpr::try_from(expr)
        )
    }
}
