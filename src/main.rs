use std::cmp::Ordering;

use anyhow::Context;

const CONTROL_GREY: &str = "\u{001b}[38;5;243m";
const CONTROL_GREEN: &str = "\u{001b}[32m";
const CONTROL_RED: &str = "\u{001b}[31m";
const CONTROL_RESET: &str = "\u{001b}[0m";

#[derive(PartialEq, Debug)]
pub struct UciOption {
    name: String,
    value: f64,
}

pub enum IOSort {
    Input,
    Output,
}

pub fn parse_from_input(text: &str, sort: IOSort) -> anyhow::Result<Vec<UciOption>> {
    text.lines()
        .enumerate()
        .map(|(i, l)| {
            let mut parts = l.split(", ");
            let name = parts
                .next()
                .with_context(|| format!("No name part in line {i}: \"{}\"", l))?;
            let val_index = match sort {
                IOSort::Input => 1,
                IOSort::Output => 0,
            };
            let val = parts
                .nth(val_index)
                .with_context(|| format!("No value part in line {i}: \"{}\"", l))?;
            Ok(UciOption {
                name: name.to_string(),
                value: val.parse()?,
            })
        })
        .collect()
}

fn main() -> anyhow::Result<()> {
    // let url = "https://chess.swehosting.se/tune/7126/";
    let url = std::env::args().nth(1).with_context(|| "No URL argument provided.")?;
    println!("fetching {url}");

    let response = minreq::get(url).send()?;
    let text = response.as_str()?;
    assert!(text.contains("</html>"));
    assert_eq!(200, response.status_code);

    let (_, rest) = text.split_once("spsa-input").with_context(|| "Did not find \"spsa-input\" in page.")?;
    let (_, rest) = rest.split_once('>').with_context(|| "Did not find end of tag after \"spsa-input\".")?;
    let (input, rest) = rest.split_once('<').with_context(|| "Did not find start of tag after SPSA input data.")?;
    let (_, rest) = rest.split_once("spsa-output").with_context(|| "Did not find \"spsa-output\" in page.")?;
    let (_, rest) = rest.split_once('>').with_context(|| "Did not find end of tag after \"spsa-output\".")?;
    let (output, _) = rest.split_once('<').with_context(|| "Did not find start of tag after SPSA output data.")?;

    // let input = include_str!("../input.txt");
    // let output = include_str!("../output.txt");
    let input = parse_from_input(input, IOSort::Input)?;
    let output = parse_from_input(output, IOSort::Output)?;

    let mut pairs = input
        .into_iter()
        .zip(output)
        .map(|p| {
            if p.0.value == 0.0 {
                return (p, 0.0);
            }
            let diff = p.1.value - p.0.value;
            let frac = diff / p.0.value.abs();
            (p, frac)
        })
        .collect::<Vec<_>>();

    pairs.sort_by(|(_, ak), (_, bk)| f64::total_cmp(&bk.abs(), &ak.abs()));

    for ((before, after), permill_change) in pairs {
        assert_eq!(before.name, after.name);
        let control = match after.value.total_cmp(&before.value) {
            Ordering::Less => CONTROL_RED,
            Ordering::Equal => CONTROL_GREY,
            Ordering::Greater => CONTROL_GREEN,
        };
        println!(
            "{:28} : {control}{:+6.1}{CONTROL_RESET}%",
            before.name,
            permill_change * 100.0,
        );
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    const EXAMPLE_INPUT: &str = r"ASPIRATION_WINDOW, int, 6.0, 1.0, 50.0, 3.0, 0.002
RFP_MARGIN, int, 73.0, 40.0, 200.0, 10.0, 0.002
RFP_IMPROVING_MARGIN, int, 58.0, 30.0, 150.0, 10.0, 0.002
DO_DEEPER_DEPTH_MARGIN, int, 11.0, 1.0, 50.0, 2.0, 0.002
HISTORY_PRUNING_DEPTH, int, 7.0, 2.0, 14.0, 1.0, 0.002
HISTORY_PRUNING_MARGIN, int, -2500.0, -5000.0, 1000.0, 500.0, 0.002";

    const EXAMPLE_OUTPUT: &str = r"ASPIRATION_WINDOW, 5
RFP_MARGIN, 73
RFP_IMPROVING_MARGIN, 58
DO_DEEPER_DEPTH_MARGIN, 11
HISTORY_PRUNING_DEPTH, 7
HISTORY_PRUNING_MARGIN, -2474";

    #[test]
    fn example_works() {
        use crate::{parse_from_input, IOSort, UciOption};

        let options = parse_from_input(EXAMPLE_INPUT, IOSort::Input).unwrap();

        assert_eq!(
            options,
            vec![
                UciOption {
                    name: "ASPIRATION_WINDOW".into(),
                    value: 6.0
                },
                UciOption {
                    name: "RFP_MARGIN".into(),
                    value: 73.0
                },
                UciOption {
                    name: "RFP_IMPROVING_MARGIN".into(),
                    value: 58.0
                },
                UciOption {
                    name: "DO_DEEPER_DEPTH_MARGIN".into(),
                    value: 11.0
                },
                UciOption {
                    name: "HISTORY_PRUNING_DEPTH".into(),
                    value: 7.0
                },
                UciOption {
                    name: "HISTORY_PRUNING_MARGIN".into(),
                    value: -2500.0
                },
            ]
        );

        let options = parse_from_input(EXAMPLE_OUTPUT, IOSort::Output).unwrap();

        assert_eq!(
            options,
            vec![
                UciOption {
                    name: "ASPIRATION_WINDOW".into(),
                    value: 5.0
                },
                UciOption {
                    name: "RFP_MARGIN".into(),
                    value: 73.0
                },
                UciOption {
                    name: "RFP_IMPROVING_MARGIN".into(),
                    value: 58.0
                },
                UciOption {
                    name: "DO_DEEPER_DEPTH_MARGIN".into(),
                    value: 11.0
                },
                UciOption {
                    name: "HISTORY_PRUNING_DEPTH".into(),
                    value: 7.0
                },
                UciOption {
                    name: "HISTORY_PRUNING_MARGIN".into(),
                    value: -2474.0
                },
            ]
        );
    }
}
