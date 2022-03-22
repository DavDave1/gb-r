use std::{borrow::Cow, str::FromStr};

use nom::{branch::alt, IResult};

pub enum Command {
    RunStop,
    Step,
    Quit,
}

impl FromStr for Command {
    type Err = Cow<'static, str>;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match command(s) {
            Ok((_, c)) => Ok(c),
            Err(e) => Err(format!("Cannot parse command: {:?}", e).into()),
        }
    }
}

fn command(input: &str) -> IResult<&str, Command> {
    alt((run_stop, step, quit))(input)
}

fn run_stop(input: &str) -> IResult<&str, Command> {
    nom::combinator::map_parser(
        alt((
            nom::bytes::complete::tag("run"),
            nom::bytes::complete::tag("r"),
        )),
        |i| Ok((i, Command::RunStop)),
    )(input)
}

fn step(input: &str) -> IResult<&str, Command> {
    nom::combinator::map_parser(
        alt((
            nom::bytes::complete::tag("step"),
            nom::bytes::complete::tag("s"),
        )),
        |i| Ok((i, Command::Step)),
    )(input)
}

fn quit(input: &str) -> IResult<&str, Command> {
    nom::combinator::map_parser(
        alt((
            nom::bytes::complete::tag("quit"),
            nom::bytes::complete::tag("exit"),
            nom::bytes::complete::tag("q"),
        )),
        |i| Ok((i, Command::Quit)),
    )(input)
}
