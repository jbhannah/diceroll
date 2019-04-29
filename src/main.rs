extern crate clap;
#[macro_use]
extern crate lazy_static;
extern crate rand;
extern crate regex;

mod dice;

use crate::dice::Dice;
use clap::{App, Arg};

fn main() {
    let matches = App::new("diceroll")
        .version("1.0")
        .author("Jesse B. Hannah <jesse@jbhannah.net>")
        .about("A command-line dice roller")
        .arg(
            Arg::with_name("EXPR")
                .help("Dice expression(s) to roll")
                .multiple(true)
                .required(true),
        )
        .get_matches();

    for expr in matches.values_of("EXPR").unwrap() {
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
