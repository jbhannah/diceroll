extern crate clap;
extern crate diceroll;

use clap::{arg, command, ArgAction};
use diceroll::expr::DiceExpr;
use std::convert::TryFrom;

fn main() {
    let matches = roll().get_matches();
    let verbose = matches.get_flag("verbose");

    for expr in matches
        .get_many::<String>("EXPR")
        .unwrap_or_default()
        .map(|v| v.as_str())
        .collect::<Vec<_>>()
    {
        let dice = match DiceExpr::try_from(expr) {
            Ok(d) => d,
            Err(e) => {
                println!("{}", e);
                continue;
            }
        };

        let (roll, rolls) = dice.roll();
        println!("{}: {}", dice, roll);

        if verbose {
            let sum: u16 = rolls.iter().sum();
            println!("Rolls: {:?} = {}\n", rolls, sum);
        }
    }
}

fn roll() -> clap::Command {
    command!("diceroll")
        .version("1.0")
        .author("Jesse B. Hannah <jesse@jbhannah.net>")
        .about("A command-line dice roller")
        .arg(
            arg!([EXPR] "Dice expression(s) to roll")
                .action(ArgAction::Append)
                .required(true),
        )
        .arg(
            arg!([verbose] "Displays details of each roll")
                .short('v')
                .long("verbose")
                .action(ArgAction::SetTrue),
        )
}

#[test]
fn verify_cli() {
    roll().debug_assert();
}
