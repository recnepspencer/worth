// The CEL intersection. Every vendored CEL conformance case is re-run as
// Worth source and must keep the disposition `cel/classification.tsv`
// records for it: `equivalent` cases are a differential oracle whose
// expected values come from CEL, `divergent` cases name the Worth rule that
// decides them and the exact Worth outcome, and `unsupported` cases record
// the stage and denial family that refuse them. An equivalent case CEL
// expects to fail also pins the Worth denial family. Set
// `WORTH_CEL_CLASSIFICATION=overwrite` to rewrite the table from the current
// outcomes; hand-written rules are kept.
pub(crate) use {std::collections::BTreeMap};
pub(crate) use {std::fmt::Write as _};

pub(crate) use {worth_foundational::expression_api::expressions, worth_foundational::expression_api::ExpressionInputs, worth_foundational::expression_api::ExpressionProfile, worth_foundational::expression_api::ExpressionSchema, worth_foundational::expression_api::ExpressionType, worth_foundational::expression_api::ExpressionValue};
pub(crate) use {worth_foundational::CanonicalF64};

pub(crate) use {crate::expressions::ordinary_context::empty_catalog};


pub(crate) use {crate::expressions::reference_context::cel::textproto::Message, crate::expressions::reference_context::cel::textproto::Node};

pub(crate) const FILES: [(&str, &str); 9] = [
    (
        "basic",
        include_str!("../../../../sources/worth_foundational/expressions/cel/basic.textproto"),
    ),
    (
        "comparisons",
        include_str!("../../../../sources/worth_foundational/expressions/cel/comparisons.textproto"),
    ),
    (
        "conversions",
        include_str!("../../../../sources/worth_foundational/expressions/cel/conversions.textproto"),
    ),
    (
        "fp_math",
        include_str!("../../../../sources/worth_foundational/expressions/cel/fp_math.textproto"),
    ),
    (
        "integer_math",
        include_str!("../../../../sources/worth_foundational/expressions/cel/integer_math.textproto"),
    ),
    (
        "lists",
        include_str!("../../../../sources/worth_foundational/expressions/cel/lists.textproto"),
    ),
    (
        "logic",
        include_str!("../../../../sources/worth_foundational/expressions/cel/logic.textproto"),
    ),
    (
        "macros",
        include_str!("../../../../sources/worth_foundational/expressions/cel/macros.textproto"),
    ),
    (
        "string",
        include_str!("../../../../sources/worth_foundational/expressions/cel/string.textproto"),
    ),
];

pub(crate) const TABLE: &str = include_str!("../../../../sources/worth_foundational/expressions/cel/classification.tsv");
pub(crate) const TABLE_PATH: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/tests/worth_court_foundational_expressions/sources/worth_foundational/expressions/cel/classification.tsv");

pub(crate) const RULES: [(&str, &str); 3] = [
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
pub(crate) enum Expected {
    Value(ExpressionValue),
    /// A CEL value Worth has no carrier for, such as `null` or `uint`.
    Foreign(String),
    Error,
}

/// What Worth does with the same source.
#[derive(Debug)]
pub(crate) enum Outcome {
    Refused(String),
    Value(ExpressionValue),
    Denied(String),
}

pub(crate) fn expected(value: &Message) -> Expected {
    let (field, node) = value.fields().next().expect("a value has one field");
    let word = || match node {
        crate::expressions::reference_context::cel::textproto::Node::Word(word) => word.as_str(),
        other => panic!("{field} is not a word: {other:?}"),
    };
    let text = || match node {
        crate::expressions::reference_context::cel::textproto::Node::Text(bytes) => bytes.clone(),
        other => panic!("{field} is not text: {other:?}"),
    };
    let message = || match node {
        crate::expressions::reference_context::cel::textproto::Node::Message(message) => message.clone(),
        other => panic!("{field} is not a message: {other:?}"),
    };
    let value = match field {
        "int64_value" => worth_foundational::expression_api::ExpressionValue::integer(word().parse().expect("int64")),
        "bool_value" => worth_foundational::expression_api::ExpressionValue::bool(word() == "true"),
        "string_value" => worth_foundational::expression_api::ExpressionValue::string(String::from_utf8(text()).expect("UTF-8")),
        "bytes_value" => worth_foundational::expression_api::ExpressionValue::bytes(text()),
        "double_value" => match word().parse::<f64>() {
            Ok(number) if number.is_finite() => {
                worth_foundational::expression_api::ExpressionValue::float64(worth_foundational::CanonicalF64::from_f64(number)).expect("finite")
            }
            _ => return crate::expressions::reference_context::cel::Expected::Foreign(format!("double {}", word())),
        },
        "list_value" => {
            let mut items = Vec::new();
            for node in message().all("values") {
                let crate::expressions::reference_context::cel::textproto::Node::Message(item) = node else {
                    panic!("list item")
                };
                match expected(item) {
                    crate::expressions::reference_context::cel::Expected::Value(value) => items.push(value),
                    other => return other,
                }
            }
            worth_foundational::expression_api::ExpressionValue::list(items)
        }
        "map_value" => {
            let mut entries = Vec::new();
            for node in message().all("entries") {
                let crate::expressions::reference_context::cel::textproto::Node::Message(entry) = node else {
                    panic!("map entry")
                };
                let key = entry.message("key").expect("entry key");
                let value = entry.message("value").expect("entry value");
                match (expected(key), expected(value)) {
                    (crate::expressions::reference_context::cel::Expected::Value(key), crate::expressions::reference_context::cel::Expected::Value(value)) => entries.push((key, value)),
                    _ => return crate::expressions::reference_context::cel::Expected::Foreign("map".to_string()),
                }
            }
            match worth_foundational::expression_api::ExpressionValue::map(entries) {
                Ok(map) => map,
                Err(_) => return crate::expressions::reference_context::cel::Expected::Foreign("map".to_string()),
            }
        }
        other => return crate::expressions::reference_context::cel::Expected::Foreign(other.trim_end_matches("_value").to_string()),
    };
    crate::expressions::reference_context::cel::Expected::Value(value)
}

pub(crate) fn primitive(ident: &Message) -> Option<ExpressionType> {
    match ident.message("type")?.word("primitive")? {
        "INT64" => Some(worth_foundational::expression_api::ExpressionType::INT64),
        "BOOL" => Some(worth_foundational::expression_api::ExpressionType::Bool),
        "DOUBLE" => Some(worth_foundational::expression_api::ExpressionType::Float64),
        "STRING" => Some(worth_foundational::expression_api::ExpressionType::String),
        "BYTES" => Some(worth_foundational::expression_api::ExpressionType::Bytes),
        _ => None,
    }
}

pub(crate) fn run(case: &Message) -> Outcome {
    let refused =
        |stage: &str, detail: &dyn std::fmt::Debug| crate::expressions::reference_context::cel::Outcome::Refused(format!("{stage}:{detail:?}"));
    let mut schema = worth_foundational::expression_api::ExpressionSchema::builder();
    for declaration in case.all("type_env") {
        let crate::expressions::reference_context::cel::textproto::Node::Message(declaration) = declaration else {
            panic!("declaration")
        };
        let name = declaration.text("name").expect("declared name");
        let Some(ty) = declaration.message("ident").and_then(primitive) else {
            return crate::expressions::reference_context::cel::Outcome::Refused("environment:Type".to_string());
        };
        schema = match schema.operand(&name, ty) {
            Ok(schema) => schema,
            Err(denial) => return refused("environment", &denial.family()),
        };
    }
    let schema = schema.build();
    let mut inputs = worth_foundational::expression_api::ExpressionInputs::builder(&schema);
    for binding in case.all("bindings") {
        let crate::expressions::reference_context::cel::textproto::Node::Message(binding) = binding else {
            panic!("binding")
        };
        let name = binding.text("key").expect("binding key");
        let value = binding
            .message("value")
            .and_then(|value| value.message("value"));
        let Some(crate::expressions::reference_context::cel::Expected::Value(value)) = value.map(expected) else {
            return crate::expressions::reference_context::cel::Outcome::Refused("environment:Value".to_string());
        };
        inputs = match inputs.bind(&name, value) {
            Ok(inputs) => inputs,
            Err(denial) => return refused("environment", &denial.family()),
        };
    }
    let source = case.text("expr").expect("a case has source");
    let profile = worth_foundational::expression_api::ExpressionProfile::interactive();
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
        Ok(value) => crate::expressions::reference_context::cel::Outcome::Value(value),
        Err(denial) => crate::expressions::reference_context::cel::Outcome::Denied(format!("{:?}", denial.family())),
    }
}

/// CEL's simple tests expect `true` when a case names no result.
pub(crate) fn expectation(case: &Message) -> Expected {
    if case.get("eval_error").is_some() || case.get("any_eval_errors").is_some() {
        crate::expressions::reference_context::cel::Expected::Error
    } else if let Some(value) = case.message("value") {
        expected(value)
    } else {
        crate::expressions::reference_context::cel::Expected::Value(worth_foundational::expression_api::ExpressionValue::bool(true))
    }
}

/// `(disposition, detail, context)`: the detail is the refusal for
/// unsupported cases and the Worth outcome otherwise; the context describes
/// CEL's expectation for failure messages.
pub(crate) fn classify(case: &Message) -> (&'static str, String, String) {
    let cel = expectation(case);
    let context = cel.describe();
    let (disposition, worth) = match (run(case), cel) {
        (crate::expressions::reference_context::cel::Outcome::Refused(refusal), _) => return ("unsupported", refusal, context),
        (crate::expressions::reference_context::cel::Outcome::Value(worth), crate::expressions::reference_context::cel::Expected::Value(cel)) if worth == cel => {
            return ("equivalent", String::new(), context)
        }
        (worth @ crate::expressions::reference_context::cel::Outcome::Denied(_), crate::expressions::reference_context::cel::Expected::Error) => ("equivalent", worth),
        (worth, _) => ("divergent", worth),
    };
    (disposition, describe(&worth), context)
}

pub(crate) fn describe(outcome: &Outcome) -> String {
    match outcome {
        crate::expressions::reference_context::cel::Outcome::Refused(refusal) => format!("refused {refusal}"),
        crate::expressions::reference_context::cel::Outcome::Value(value) => format!("{value:?}"),
        crate::expressions::reference_context::cel::Outcome::Denied(family) => format!("denied {family}"),
    }
}

impl Expected {
    pub(crate) fn describe(&self) -> String {
        match self {
            Self::Value(value) => format!("{value:?}"),
            Self::Foreign(value) => format!("CEL-only {value}"),
            Self::Error => "error".to_string(),
        }
    }
}

/// Every case keyed `file/section/test`, in file order.
pub(crate) fn cases() -> Vec<(String, Message)> {
    let mut cases = Vec::new();
    for (file, source) in FILES {
        let root = crate::expressions::reference_context::cel::textproto::parse(source);
        for section in root.all("section") {
            let crate::expressions::reference_context::cel::textproto::Node::Message(section) = section else {
                panic!("section")
            };
            let section_name = section.text("name").expect("section name");
            for case in section.all("test") {
                let crate::expressions::reference_context::cel::textproto::Node::Message(case) = case else {
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
pub(crate) fn recorded() -> BTreeMap<String, (String, String)> {
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