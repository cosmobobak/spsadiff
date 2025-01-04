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
    min: Option<f64>,
    max: Option<f64>,
    step: Option<f64>,
}

pub enum IOSort {
    Input,
    Output,
}

// example output:
// RFP_MARGIN, 73
// example input:
// RFP_MARGIN, int, 73.0, 40.0, 200.0, 10.0, 0.002

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
            let min = parts.next().and_then(|s| s.parse().ok());
            let max = parts.next().and_then(|s| s.parse().ok());
            let step = parts.next().and_then(|s| s.parse().ok());
            Ok(UciOption {
                name: name.to_string(),
                value: val.parse()?,
                min,
                max,
                step,
            })
        })
        .collect()
}

fn main() -> anyhow::Result<()> {
    // let url = "https://chess.swehosting.se/tune/7126/";
    let url = std::env::args()
        .nth(1)
        .with_context(|| "NO URL ARGUMENT PROVIDED")?;
    println!("FETCHING {url}");

    let response = minreq::get(url).send()?;
    let text = response.as_str()?;
    anyhow::ensure!(
        text.contains("</html>"),
        "HTML CLOSING TAG NOT FOUND IN TEXT"
    );
    anyhow::ensure!(
        200 == response.status_code,
        "RESPONSE 200 OK NOT FOUND: {}",
        response.status_code
    );

    let (_, rest) = text
        .split_once("spsa-input")
        .with_context(|| "Did not find \"spsa-input\" in page.")?;
    let (_, rest) = rest
        .split_once('>')
        .with_context(|| "Did not find end of tag after \"spsa-input\".")?;
    let (input, rest) = rest
        .split_once('<')
        .with_context(|| "Did not find start of tag after SPSA input data.")?;
    let (_, rest) = rest
        .split_once("spsa-output")
        .with_context(|| "Did not find \"spsa-output\" in page.")?;
    let (_, rest) = rest
        .split_once('>')
        .with_context(|| "Did not find end of tag after \"spsa-output\".")?;
    let (output, _) = rest
        .split_once('<')
        .with_context(|| "Did not find start of tag after SPSA output data.")?;

    // let input = include_str!("../input.txt");
    // let output = include_str!("../output.txt");
    let input = parse_from_input(input, IOSort::Input)?;
    let output = parse_from_input(output, IOSort::Output)?;

    let mut pairs = input
        .into_iter()
        .zip(output)
        .map(|p| {
            let range = p.0.max.unwrap_or(f64::INFINITY) - p.0.min.unwrap_or(f64::NEG_INFINITY);
            let diff = p.1.value - p.0.value;
            let frac = diff / range;
            (p, frac)
        })
        .collect::<Vec<_>>();

    pairs.sort_by(|(_, ak), (_, bk)| f64::total_cmp(&bk.abs(), &ak.abs()));

    let line_width = 45;
    println!();
    println!(
        "OPTION NAME {pad} CHANGE",
        pad = " ".repeat(line_width - 20)
    );
    println!("{}", "-".repeat(line_width + 5));
    for ((before, after), _) in pairs {
        assert_eq!(before.name, after.name);
        let control = match after.value.total_cmp(&before.value) {
            Ordering::Less => CONTROL_RED,
            Ordering::Equal => CONTROL_GREY,
            Ordering::Greater => CONTROL_GREEN,
        };
        println!(
            "{} {pad} {before} -> {control}{after}{CONTROL_RESET} {tail}",
            before.name,
            pad = ".".repeat(36usize.saturating_sub(before.name.len() + before.value.abs().log10() as usize + usize::from(before.value < 0.0))
            ),
            before = before.value,
            after = after.value,
            tail = ".".repeat(5usize.saturating_sub(after.value.abs().log10() as usize + usize::from(after.value < 0.0)))
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
                    value: 6.0,
                    min: Some(1.0),
                    max: Some(50.0),
                    step: Some(3.0),
                },
                UciOption {
                    name: "RFP_MARGIN".into(),
                    value: 73.0,
                    min: Some(40.0),
                    max: Some(200.0),
                    step: Some(10.0),
                },
                UciOption {
                    name: "RFP_IMPROVING_MARGIN".into(),
                    value: 58.0,
                    min: Some(30.0),
                    max: Some(150.0),
                    step: Some(10.0),
                },
                UciOption {
                    name: "DO_DEEPER_DEPTH_MARGIN".into(),
                    value: 11.0,
                    min: Some(1.0),
                    max: Some(50.0),
                    step: Some(2.0),
                },
                UciOption {
                    name: "HISTORY_PRUNING_DEPTH".into(),
                    value: 7.0,
                    min: Some(2.0),
                    max: Some(14.0),
                    step: Some(1.0),
                },
                UciOption {
                    name: "HISTORY_PRUNING_MARGIN".into(),
                    value: -2500.0,
                    min: Some(-5000.0),
                    max: Some(1000.0),
                    step: Some(500.0),
                },
            ]
        );

        let options = parse_from_input(EXAMPLE_OUTPUT, IOSort::Output).unwrap();

        assert_eq!(
            options,
            vec![
                UciOption {
                    name: "ASPIRATION_WINDOW".into(),
                    value: 5.0,
                    min: None,
                    max: None,
                    step: None,
                },
                UciOption {
                    name: "RFP_MARGIN".into(),
                    value: 73.0,
                    min: None,
                    max: None,
                    step: None,
                },
                UciOption {
                    name: "RFP_IMPROVING_MARGIN".into(),
                    value: 58.0,
                    min: None,
                    max: None,
                    step: None,
                },
                UciOption {
                    name: "DO_DEEPER_DEPTH_MARGIN".into(),
                    value: 11.0,
                    min: None,
                    max: None,
                    step: None,
                },
                UciOption {
                    name: "HISTORY_PRUNING_DEPTH".into(),
                    value: 7.0,
                    min: None,
                    max: None,
                    step: None,
                },
                UciOption {
                    name: "HISTORY_PRUNING_MARGIN".into(),
                    value: -2474.0,
                    min: None,
                    max: None,
                    step: None,
                },
            ]
        );
    }
}
