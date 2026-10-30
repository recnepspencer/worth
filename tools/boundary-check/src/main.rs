mod config;
mod document_rules;
mod graph_rules;
mod inputs;
mod lexical;
mod runtime_courts;
mod seal_rules;
mod text_rules;
use config::{diagnostic, Diagnostic};
use inputs::Inputs;
use std::{env, path::PathBuf, thread};

fn main() {
    let mut root = PathBuf::from(".");
    let mut config = format!("{}road1.toml", config::CONFIG);
    let mut json = false;
    let mut args = env::args().skip(1);
    while let Some(arg) = args.next() {
        match (arg.as_str(), args.next()) {
            ("--root", Some(value)) => root = value.into(),
            ("--config", Some(value)) => config = value,
            ("--format", Some(value)) if value == "json" || value == "human" => {
                json = value == "json"
            }
            _ => {
                eprintln!(
                    "usage: boundary-check [--root PATH] [--config PATH] [--format human|json]"
                );
                std::process::exit(2);
            }
        }
    }
    let result = root
        .canonicalize()
        .map_err(|e| e.to_string())
        .and_then(|root| Inputs::read(&root, &config));
    let outcome = match result {
        Ok(input) => thread::scope(|s| {
            let rules: [fn(&Inputs) -> Vec<Diagnostic>; 5] = [
                graph_rules::check,
                text_rules::check,
                document_rules::check,
                seal_rules::check,
                runtime_courts::check,
            ];
            let handles: Vec<_> = rules
                .into_iter()
                .map(|rule| {
                    let input = &input;
                    s.spawn(move || rule(input))
                })
                .collect();
            handles
                .into_iter()
                .flat_map(|h| h.join().expect("rule panicked"))
                .collect::<Vec<_>>()
        }),
        Err(error) => return emit(Outcome::InputFailure(error), json),
    };
    emit(Outcome::Checked(outcome), json);
}
enum Outcome {
    Checked(Vec<Diagnostic>),
    InputFailure(String),
}
impl Outcome {
    fn report(self) -> (Vec<Diagnostic>, i32) {
        match self {
            Self::Checked(mut diagnostics) => {
                diagnostics.sort();
                diagnostics.dedup();
                let status = i32::from(!diagnostics.is_empty());
                (diagnostics, status)
            }
            Self::InputFailure(error) => (
                vec![diagnostic("BOUNDARY_INPUT_FAILURE", "inputs", error)],
                2,
            ),
        }
    }
}
fn emit(outcome: Outcome, json: bool) {
    let (diagnostics, status) = outcome.report();
    if json {
        println!("{}", serde_json::to_string_pretty(&diagnostics).unwrap());
    } else if diagnostics.is_empty() {
        println!("boundary-check: repository boundaries are valid");
    } else {
        for d in &diagnostics {
            eprintln!(
                "{} {}: {}\nbelongs: {}",
                d.code, d.subject, d.message, d.legal_home
            );
        }
    }
    if status != 0 {
        std::process::exit(status);
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn unavailable_inputs_are_distinct_from_checked_dependency_violations() {
        let (unchecked, status) = Outcome::InputFailure("graph unavailable".into()).report();
        assert_eq!(status, 2);
        let json: serde_json::Value =
            serde_json::from_str(&serde_json::to_string(&unchecked).unwrap()).unwrap();
        assert_eq!(json[0]["code"], "BOUNDARY_INPUT_FAILURE");
        let (checked, status) =
            Outcome::Checked(vec![diagnostic("BC2001", "crate", "forbidden dependency")]).report();
        assert_eq!(status, 1);
        assert_eq!(checked[0].code, "BC2001");
        assert_eq!(Outcome::Checked(Vec::new()).report().1, 0);
    }
}
