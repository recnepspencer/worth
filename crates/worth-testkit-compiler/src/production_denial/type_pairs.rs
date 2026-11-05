//! Canonical type notes pair each required type with its supplied type.
use serde_json::Value;

use super::{assignment, ProductionTypeMismatchPair};

pub(super) fn matches(error: &Value, expected: &[ProductionTypeMismatchPair<'_>]) -> bool {
    // rustc emits full, unspanned type notes once per mismatching argument.
    // Spanned abbreviated notes and labels repeat those same observations.
    let mut actual = Vec::new();
    for child in error["children"].as_array().into_iter().flatten() {
        if child["level"] == "note" && child["spans"].as_array().is_some_and(Vec::is_empty) {
            actual.extend(pairs(child["message"].as_str().unwrap_or_default()));
        }
    }
    if actual.is_empty() {
        for span in error["spans"].as_array().into_iter().flatten() {
            actual.extend(pairs(span["label"].as_str().unwrap_or_default()));
        }
    }
    if actual.is_empty() {
        actual = pairs(error["message"].as_str().unwrap_or_default());
    }
    let candidates = expected
        .iter()
        .map(|required| {
            actual
                .iter()
                .enumerate()
                .filter(|(_, (wanted, supplied))| {
                    same_type(required.expected, wanted) && same_type(required.found, supplied)
                })
                .map(|(index, _)| index)
                .collect::<Vec<_>>()
        })
        .collect::<Vec<_>>();
    assignment::covers_all(&candidates, actual.len())
}

fn pairs(text: &str) -> Vec<(String, String)> {
    let mut result = Vec::new();
    let mut expected = None;
    let mut remaining = text;
    while let Some((prefix, quoted)) = remaining.split_once('`') {
        let Some((value, tail)) = quoted.split_once('`') else {
            break;
        };
        let role = prefix.lines().next_back().unwrap_or_default().trim();
        if role_suffix(role, "expected") || category_after(role, "expected") {
            expected = Some(value.to_owned());
        } else if (role_suffix(role, "found") || category_after(role, "found"))
            && let Some(expected) = expected.take()
        {
            result.push((expected, value.to_owned()));
        }
        remaining = tail;
    }
    result
}

fn category_after(text: &str, role: &str) -> bool {
    [
        "struct",
        "enum",
        "reference",
        "array",
        "slice",
        "tuple",
        "type",
    ]
    .iter()
    .any(|category| role_suffix(text, &format!("{role} {category}")))
}

fn role_suffix(text: &str, suffix: &str) -> bool {
    text.strip_suffix(suffix).is_some_and(|prefix| {
        prefix
            .chars()
            .next_back()
            .is_none_or(|last| !last.is_alphanumeric() && last != '_')
    })
}

fn same_type(expected: &str, actual: &str) -> bool {
    let left = tokens(expected);
    let right = tokens(actual);
    left.len() == right.len()
        && left.iter().zip(right).all(|(left, right)| {
            left == &right
                || (!left.contains("::") || !right.contains("::"))
                    && left.rsplit("::").next() == right.rsplit("::").next()
        })
}

fn tokens(text: &str) -> Vec<String> {
    let mut result = Vec::new();
    let mut chars = text.chars().peekable();
    while let Some(character) = chars.next() {
        if character.is_whitespace() {
            continue;
        }
        let mut token = character.to_string();
        if character.is_alphabetic() || character == '_' {
            loop {
                while chars
                    .peek()
                    .is_some_and(|next| next.is_alphanumeric() || *next == '_')
                {
                    token.push(chars.next().expect("peeked identifier character"));
                }
                let mut qualifier = chars.clone();
                if qualifier.next() != Some(':')
                    || qualifier.next() != Some(':')
                    || !qualifier
                        .peek()
                        .is_some_and(|next| next.is_alphabetic() || *next == '_')
                {
                    break;
                }
                chars.next();
                chars.next();
                token.push_str("::");
                token.push(chars.next().expect("peeked qualified identifier"));
            }
        }
        result.push(token);
    }
    result
}
