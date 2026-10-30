//! Literal exported declarations and the bodies of receiver-owned test doors.
use crate::lexical::{close, Token};
use std::collections::BTreeSet;
pub fn test_name(name: &str) -> bool {
    name.ends_with("_for_test") || name.ends_with("ForTest")
}
fn boundary(t: &[Token<'_>], at: usize) -> usize {
    t[..at]
        .iter()
        .rposition(|t| [";", "{", "}"].contains(&t.text))
        .map_or(0, |i| i + 1)
}
fn exported(t: &[Token<'_>]) -> bool {
    let Some(i) = t.iter().rposition(|t| t.text == "pub") else {
        return false;
    };
    t.get(i + 1).is_none_or(|t| t.text != "(")
        || t.get(i + 2).is_some_and(|t| t.text == "crate")
            && t.get(i + 3).is_some_and(|t| t.text == ")")
}
fn test_only(t: &[Token<'_>]) -> bool {
    t.windows(7).any(|w| {
        w.iter()
            .zip(["#", "[", "cfg", "(", "test", ")", "]"])
            .all(|(t, expected)| t.text == expected)
    })
}
pub fn test_items(t: &[Token<'_>]) -> Vec<(String, usize, usize)> {
    (0..t.len().saturating_sub(7))
        .filter(|i| test_only(&t[*i..*i + 7]))
        .map(|i| ("item".into(), t[i].start, t[end(t, i + 7)].end))
        .collect()
}
fn end(t: &[Token<'_>], start: usize) -> usize {
    let mut at = start;
    while at < t.len() {
        match t[at].text {
            "(" | "[" => at = close(t, at) + 1,
            ";" | "{" => return close(t, at),
            _ => at += 1,
        }
    }
    t.len() - 1
}
fn leaves<'a>(tokens: &[Token<'a>]) -> Vec<&'a str> {
    tokens
        .iter()
        .enumerate()
        .filter(|(i, token)| {
            token
                .text
                .chars()
                .next()
                .is_some_and(|c| c.is_alphabetic() || c == '_')
                && !["as", "self", "super", "crate", "r"].contains(&token.text)
                && !tokens
                    .get(i + 1)
                    .is_some_and(|t| [":", "as"].contains(&t.text))
        })
        .map(|(_, t)| t.text)
        .collect()
}
pub fn declared(t: &[Token<'_>], test_ranges: &[(String, usize, usize)]) -> BTreeSet<String> {
    let mut names = BTreeSet::new();
    let bodies = function_bodies(t);
    for i in 0..t.len().saturating_sub(1) {
        if ![
            "pub", "fn", "struct", "enum", "trait", "type", "mod", "const", "static", "use",
        ]
        .contains(&t[i].text)
        {
            continue;
        }
        if bodies.iter().any(|(a, b)| i > *a && i < *b) {
            continue;
        }
        if t[i].text == "pub" {
            let next = if t[i + 1].text == "(" {
                close(t, i + 1) + 1
            } else {
                i + 1
            };
            if t.get(next).is_some_and(|t| test_name(t.text))
                && exported(&t[i..next])
                && !test_only(&t[boundary(t, i)..i])
                && !test_ranges
                    .iter()
                    .any(|(_, a, b)| t[i].start >= *a && t[i].start < *b)
            {
                names.insert(t[next].text.into());
            }
        }
        if ![
            "fn", "struct", "enum", "trait", "type", "mod", "const", "static", "use",
        ]
        .contains(&t[i].text)
        {
            continue;
        }
        let prefix = &t[boundary(t, i)..i];
        if !exported(prefix)
            || test_only(prefix)
            || test_ranges
                .iter()
                .any(|(_, a, b)| t[i].start >= *a && t[i].start < *b)
        {
            continue;
        }
        if t[i].text == "use" {
            for name in leaves(&t[i + 1..=end(t, i)]) {
                if test_name(name) {
                    names.insert(name.into());
                }
            }
        } else if test_name(t[i + 1].text) {
            names.insert(t[i + 1].text.into());
        }
        if t[i].text == "trait" {
            for w in t[i..=end(t, i)].windows(2) {
                if ["fn", "const", "type"].contains(&w[0].text) && test_name(w[1].text) {
                    names.insert(w[1].text.into());
                }
            }
        }
    }
    names
}
fn function_bodies(t: &[Token<'_>]) -> Vec<(usize, usize)> {
    t.iter()
        .enumerate()
        .filter(|(_, t)| t.text == "fn")
        .filter_map(|(i, _)| {
            let end = end(t, i);
            if t[end].text != "}" {
                return None;
            }
            let start = (i..end).find(|i| t[*i].text == "{")?;
            Some((start, end))
        })
        .collect()
}
pub fn ambiguous_macro(
    t: &[Token<'_>],
    declarations: &BTreeSet<String>,
    tests: &[(String, usize, usize)],
) -> bool {
    let bodies = function_bodies(t);
    for i in 1..t.len().saturating_sub(1) {
        if t[i].text != "!"
            || !["(", "{", "["].contains(&t[i + 1].text)
            || t[i - 1].text == "thread_local"
            || bodies.iter().any(|(a, b)| i > *a && i < *b)
            || tests
                .iter()
                .any(|(_, a, b)| t[i].start >= *a && t[i].start < *b)
        {
            continue;
        }
        if t[i + 1..=close(t, i + 1)]
            .iter()
            .any(|t| test_name(t.text) && !declarations.contains(t.text))
        {
            return true;
        }
    }
    false
}
pub fn ordinary_ranges(t: &[Token<'_>]) -> Vec<(usize, usize)> {
    let mut ranges = Vec::new();
    for i in 0..t.len().saturating_sub(1) {
        if !["fn", "struct", "enum", "type", "mod", "impl", "use"].contains(&t[i].text) {
            continue;
        }
        let start = boundary(t, i);
        let end = end(t, i);
        let prefix = &t[start..i];
        let control = if t[i].text == "impl" {
            t[i..=end]
                .iter()
                .take_while(|t| t.text != "{")
                .any(|t| test_name(t.text))
        } else if t[i].text == "use" {
            let leaves = leaves(&t[i + 1..=end]);
            !leaves.is_empty() && leaves.iter().all(|n| n.ends_with("ForTest"))
        } else {
            test_name(t[i + 1].text)
                && (t[i].text != "fn" || prefix.iter().any(|t| t.text == "pub"))
        };
        if control || test_only(prefix) {
            ranges.push((t[start].start, t[end].end));
        }
    }
    ranges
}
