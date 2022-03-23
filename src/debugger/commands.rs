use std::{borrow::Cow, str::FromStr};

use nom::{branch::alt, IResult};

#[derive(Debug, PartialEq)]
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

#[cfg(test)]
mod tests {
    use super::Command;

    #[test]
    fn parse_run_stop_command() {
        let c = "run".parse::<Command>();

        assert!(c.is_ok());
        assert_eq!(c.unwrap(), Command::RunStop);

        let c = "ren".parse::<Command>();

        assert!(c.is_err());
    }
}
