//! The CEL intersection. Every vendored CEL conformance case is re-run as
//! Worth source and must keep the disposition `cel/classification.tsv`
//! records for it: `equivalent` cases are a differential oracle whose
//! expected values come from CEL, `divergent` cases name the Worth rule that
//! decides them, and `unsupported` cases record the stage and denial family
//! that refuse them. Set `WORTH_CEL_CLASSIFICATION=overwrite` to rewrite the
//! table from the current outcomes; hand-written rules are kept.

use std::collections::BTreeMap;
use std::fmt::Write as _;

use worth_foundational::expression_api::{
    expressions, ExpressionInputs, ExpressionProfile, ExpressionSchema, ExpressionType,
    ExpressionValue,
};
use worth_foundational::CanonicalF64;

use super::empty_catalog;

#[path = "cel/textproto.rs"]
mod textproto;

use textproto::{Message, Node};

const FILES: [(&str, &str); 9] = [
    ("basic", include_str!("cel/basic.textproto")),
    ("comparisons", include_str!("cel/comparisons.textproto")),
    ("conversions", include_str!("cel/conversions.textproto")),
    ("fp_math", include_str!("cel/fp_math.textproto")),
    ("integer_math", include_str!("cel/integer_math.textproto")),
    ("lists", include_str!("cel/lists.textproto")),
    ("logic", include_str!("cel/logic.textproto")),
    ("macros", include_str!("cel/macros.textproto")),
    ("string", include_str!("cel/string.textproto")),
];

/// The table is compiled in, so the suite also runs where the test has no
/// file system, such as wasm32; only an overwrite touches the file.
const TABLE: &str = include_str!("cel/classification.tsv");
const TABLE_PATH: &str = concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/tests/expressions/cel/classification.tsv"
);

/// The Worth rules that decide divergent cases.
const RULES: [(&str, &str); 3] = [
    (
        "finite-floats",
        "Float results are finite: NaN, infinity, and division by zero deny.",
    ),
    (
        "left-to-right-errors",
        "Errors propagate left to right; a later operand never absorbs an earlier failure.",
    ),
    (
        "no-null",
        "Worth has no null; `null` is an ordinary operand name, and absence is `Option`.",
    ),
];

/// What CEL expects of a case.
#[derive(Debug)]
enum Expected {
    Value(ExpressionValue),
    /// A CEL value Worth has no carrier for, such as `null` or `uint`.
    Foreign(String),
    Error,
}

/// What Worth does with the same source.
#[derive(Debug)]
enum Outcome {
    Refused(String),
    Value(ExpressionValue),
    Denied(String),
}

fn expected(value: &Message) -> Expected {
    let (field, node) = value.fields().next().expect("a value has one field");
    let word = || match node {
        Node::Word(word) => word.as_str(),
        other => panic!("{field} is not a word: {other:?}"),
    };
    let text = || match node {
        Node::Text(bytes) => bytes.clone(),
        other => panic!("{field} is not text: {other:?}"),
    };
    let message = || match node {
        Node::Message(message) => message.clone(),
        other => panic!("{field} is not a message: {other:?}"),
    };
    let value = match field {
        "int64_value" => ExpressionValue::integer(word().parse().expect("int64")),
        "bool_value" => ExpressionValue::bool(word() == "true"),
        "string_value" => ExpressionValue::string(String::from_utf8(text()).expect("UTF-8")),
        "bytes_value" => ExpressionValue::bytes(text()),
        "double_value" => match word().parse::<f64>() {
            Ok(number) if number.is_finite() => {
                ExpressionValue::float64(CanonicalF64::from_f64(number)).expect("finite")
            }
            _ => return Expected::Foreign(format!("double {}", word())),
        },
        "list_value" => {
            let mut items = Vec::new();
            for node in message().all("values") {
                let Node::Message(item) = node else {
                    panic!("list item")
                };
                match expected(item) {
                    Expected::Value(value) => items.push(value),
                    other => return other,
                }
            }
            ExpressionValue::list(items)
        }
        "map_value" => {
            let mut entries = Vec::new();
            for node in message().all("entries") {
                let Node::Message(entry) = node else {
                    panic!("map entry")
                };
                let key = entry.message("key").expect("entry key");
                let value = entry.message("value").expect("entry value");
                match (expected(key), expected(value)) {
                    (Expected::Value(key), Expected::Value(value)) => entries.push((key, value)),
                    _ => return Expected::Foreign("map".to_string()),
                }
            }
            match ExpressionValue::map(entries) {
                Ok(map) => map,
                Err(_) => return Expected::Foreign("map".to_string()),
            }
        }
        other => return Expected::Foreign(other.trim_end_matches("_value").to_string()),
    };
    Expected::Value(value)
}

fn primitive(ident: &Message) -> Option<ExpressionType> {
    match ident.message("type")?.word("primitive")? {
        "INT64" => Some(ExpressionType::INT64),
        "BOOL" => Some(ExpressionType::Bool),
        "DOUBLE" => Some(ExpressionType::Float64),
        "STRING" => Some(ExpressionType::String),
        "BYTES" => Some(ExpressionType::Bytes),
        _ => None,
    }
}

fn run(case: &Message) -> Outcome {
    let refused =
        |stage: &str, detail: &dyn std::fmt::Debug| Outcome::Refused(format!("{stage}:{detail:?}"));
    let mut schema = ExpressionSchema::builder();
    for declaration in case.all("type_env") {
        let Node::Message(declaration) = declaration else {
            panic!("declaration")
        };
        let name = declaration.text("name").expect("declared name");
        let Some(ty) = declaration.message("ident").and_then(primitive) else {
            return Outcome::Refused("environment:Type".to_string());
        };
        schema = match schema.operand(&name, ty) {
            Ok(schema) => schema,
            Err(denial) => return refused("environment", &denial.family()),
        };
    }
    let schema = schema.build();
    let mut inputs = ExpressionInputs::builder(&schema);
    for binding in case.all("bindings") {
        let Node::Message(binding) = binding else {
            panic!("binding")
        };
        let name = binding.text("key").expect("binding key");
        let value = binding
            .message("value")
            .and_then(|value| value.message("value"));
        let Some(Expected::Value(value)) = value.map(expected) else {
            return Outcome::Refused("environment:Value".to_string());
        };
        inputs = match inputs.bind(&name, value) {
            Ok(inputs) => inputs,
            Err(denial) => return refused("environment", &denial.family()),
        };
    }
    let source = case.text("expr").expect("a case has source");
    let profile = ExpressionProfile::interactive();
    let draft = match expressions().parse(&source) {
        Ok(draft) => draft,
        Err(denial) => return refused("parse", &denial.family()),
    };
    let admitted = match draft.admit(&schema, &empty_catalog(&schema), profile) {
        Ok(admitted) => admitted,
        Err(denial) => return refused("admission", &denial.family()),
    };
    match admitted
        .compile()
        .evaluate(&inputs.build(), &profile)
        .into_result()
    {
        Ok(value) => Outcome::Value(value),
        Err(denial) => Outcome::Denied(format!("{:?}", denial.family())),
    }
}

/// CEL's simple tests expect `true` when a case names no result.
fn expectation(case: &Message) -> Expected {
    if case.get("eval_error").is_some() || case.get("any_eval_errors").is_some() {
        Expected::Error
    } else if let Some(value) = case.message("value") {
        expected(value)
    } else {
        Expected::Value(ExpressionValue::bool(true))
    }
}

/// `(disposition, detail)`: the refusal for unsupported cases, a
/// description of both outcomes for divergent ones.
fn classify(case: &Message) -> (&'static str, String) {
    match (run(case), expectation(case)) {
        (Outcome::Refused(refusal), _) => ("unsupported", refusal),
        (Outcome::Value(worth), Expected::Value(cel)) if worth == cel => {
            ("equivalent", String::new())
        }
        (Outcome::Denied(_), Expected::Error) => ("equivalent", String::new()),
        (worth, cel) => (
            "divergent",
            format!("worth {}, cel {}", describe(&worth), cel.describe()),
        ),
    }
}

fn describe(outcome: &Outcome) -> String {
    match outcome {
        Outcome::Refused(refusal) => format!("refused {refusal}"),
        Outcome::Value(value) => format!("{value:?}"),
        Outcome::Denied(family) => format!("denied {family}"),
    }
}

impl Expected {
    fn describe(&self) -> String {
        match self {
            Self::Value(value) => format!("{value:?}"),
            Self::Foreign(value) => format!("CEL-only {value}"),
            Self::Error => "error".to_string(),
        }
    }
}

/// Every case keyed `file/section/test`, in file order.
fn cases() -> Vec<(String, Message)> {
    let mut cases = Vec::new();
    for (file, source) in FILES {
        let root = textproto::parse(source);
        for section in root.all("section") {
            let Node::Message(section) = section else {
                panic!("section")
            };
            let section_name = section.text("name").expect("section name");
            for case in section.all("test") {
                let Node::Message(case) = case else {
                    panic!("test")
                };
                let name = case.text("name").expect("test name");
                cases.push((format!("{file}/{section_name}/{name}"), case.clone()));
            }
        }
    }
    cases
}

/// The recorded table: key to `(disposition, note)`.
fn recorded() -> BTreeMap<String, (String, String)> {
    TABLE
        .lines()
        .filter(|line| !line.starts_with('#') && !line.is_empty())
        .map(|line| {
            let mut columns = line.split('\t');
            let key = columns.next().expect("key").to_string();
            let disposition = columns.next().expect("disposition").to_string();
            (key, (disposition, columns.next().unwrap_or("").to_string()))
        })
        .collect()
}

#[test]
fn every_cel_case_keeps_its_recorded_disposition() {
    let recorded = recorded();
    let overwrite = std::env::var("WORTH_CEL_CLASSIFICATION").as_deref() == Ok("overwrite");
    let mut table = String::from("# case\tdisposition\trefusal or governing rule\n");
    let mut failures = Vec::new();
    let mut counts = BTreeMap::<&str, usize>::new();
    let cases = cases();
    for (key, case) in &cases {
        let (disposition, detail) = classify(case);
        *counts.entry(disposition).or_default() += 1;
        let previous = recorded.get(key);
        let note = match disposition {
            "divergent" => {
                let rule = previous
                    .filter(|(recorded, _)| recorded == "divergent")
                    .map(|(_, rule)| rule.clone())
                    .unwrap_or_default();
                if !RULES.iter().any(|(name, _)| *name == rule) {
                    failures.push(format!("{key}: divergent without a known rule; {detail}"));
                }
                rule
            }
            _ => detail,
        };
        let row = (disposition.to_string(), note.clone());
        if previous != Some(&row) {
            failures.push(format!("{key}: recorded {previous:?}, now {row:?}"));
        }
        writeln!(table, "{key}\t{disposition}\t{note}").expect("string write");
    }
    let known: std::collections::BTreeSet<_> = cases.iter().map(|(key, _)| key).collect();
    assert_eq!(known.len(), cases.len(), "case keys are unique");
    for key in recorded.keys().filter(|key| !known.contains(key)) {
        failures.push(format!("{key}: recorded but no longer vendored"));
    }
    if overwrite {
        std::fs::write(TABLE_PATH, table).expect("the table is writable");
    }
    let equivalent = counts.get("equivalent").copied().unwrap_or_default();
    assert!(
        equivalent >= 100,
        "the intersection is substantial: {counts:?}"
    );
    assert!(failures.is_empty(), "{counts:?}\n{}", failures.join("\n"));
}
