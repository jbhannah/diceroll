use mockall::automock;

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
        rng.gen_range(1..=self.sides)
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
