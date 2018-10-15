#[macro_use]
extern crate lazy_static;
extern crate rand;
extern crate regex;

mod dice;

use dice::Dice;
use std::env::args;

fn main() {
    for expr in args().skip(1) {
        let dice = match Dice::new(&expr) {
            Ok(d) => d,
            Err(e) => {
                println!("{}", e);
                continue;
            }
        };

        println!("{}: {}", dice.expr(), dice.roll());
    }
}
