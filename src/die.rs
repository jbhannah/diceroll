use mockall::automock;
use rand::distributions::{Distribution, Uniform};

#[derive(PartialEq, Debug)]
pub struct Die {
    sides: u16,
}

#[automock]
impl Die {
    pub fn new(sides: u16) -> Self {
        Die { sides }
    }

    pub fn roll<R: rand::Rng + 'static>(&self, rng: &mut R) -> u16 {
        Uniform::from(1..=self.sides).sample(rng)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rand::rngs::mock::StepRng;

    #[test]
    fn test_roll() {
        let mut rng = StepRng::new(1, 0);
        let die = Die::new(4);
        assert_eq!(die.roll(&mut rng), 1);
    }
}
