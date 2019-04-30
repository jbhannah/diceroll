extern crate clap;
extern crate diceroll;

use diceroll::Dice;
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
        .arg(
            Arg::with_name("verbose")
                .long("verbose")
                .short("v")
                .help("Displays details of each roll"),
        )
        .get_matches();
    let verbose = matches.is_present("verbose");

    for expr in matches.values_of("EXPR").unwrap() {
        let dice = match Dice::new(&expr) {
            Ok(d) => d,
            Err(e) => {
                println!("{}", e);
                continue;
            }
        };

        let (roll, rolls) = dice.roll();
        println!("{}: {}", dice, roll);

        if verbose {
            println!("Rolls: {:?}\n", rolls);
        }
    }
}
