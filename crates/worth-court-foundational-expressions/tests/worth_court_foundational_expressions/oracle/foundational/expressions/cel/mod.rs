// The CEL intersection. Every vendored CEL conformance case is re-run as
// Worth source and must keep the disposition `cel/classification.tsv`
// records for it: `equivalent` cases are a differential oracle whose
// expected values come from CEL, `divergent` cases name the Worth rule that
// decides them and the exact Worth outcome, and `unsupported` cases record
// the stage and denial family that refuse them. An equivalent case CEL
// expects to fail also pins the Worth denial family. Set
// `WORTH_CEL_CLASSIFICATION=overwrite` to rewrite the table from the current
// outcomes; hand-written rules are kept.
#[allow(unused_imports)]
use crate::expressions::reference_context::cel::*;
pub(crate) use std::collections::BTreeMap;
pub(crate) use std::fmt::Write as _;

pub(crate) use worth_foundational::CanonicalF64;
pub(crate) use {
    worth_foundational::expression_api::expressions,
    worth_foundational::expression_api::ExpressionInputs,
    worth_foundational::expression_api::ExpressionProfile,
    worth_foundational::expression_api::ExpressionSchema,
    worth_foundational::expression_api::ExpressionType,
    worth_foundational::expression_api::ExpressionValue,
};

pub(crate) use crate::expressions::ordinary_context::empty_catalog;

pub(crate) use {
    crate::expressions::reference_context::cel::textproto::Message,
    crate::expressions::reference_context::cel::textproto::Node,
};

#[test]
fn every_cel_case_keeps_its_recorded_disposition() {
    let recorded = recorded();
    let overwrite = std::env::var("WORTH_CEL_CLASSIFICATION").as_deref() == Ok("overwrite");
    let mut table =
        String::from("# case\tdisposition\trefusal, Worth denial, or rule -> Worth outcome\n");
    let mut failures = Vec::new();
    let mut counts = BTreeMap::<&str, usize>::new();
    let cases = cases();
    for (key, case) in &cases {
        let (disposition, detail, context) = classify(case);
        *counts.entry(disposition).or_default() += 1;
        let previous = recorded.get(key);
        let note = match disposition {
            "divergent" => {
                let rule = previous
                    .filter(|(recorded, _)| recorded == "divergent")
                    .and_then(|(_, note)| note.split(" -> ").next())
                    .unwrap_or_default()
                    .to_string();
                if !RULES.iter().any(|(name, _)| *name == rule) {
                    failures.push(format!("{key}: divergent without a known rule"));
                }
                format!("{rule} -> {detail}")
            }
            _ => detail,
        };
        let row = (disposition.to_string(), note.clone());
        if previous != Some(&row) {
            failures.push(format!(
                "{key}: recorded {previous:?}, now {row:?}; cel {context}"
            ));
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
