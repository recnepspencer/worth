//! Actual enum variants are values; non-enum shadowing stays conservative.
use super::*;
use crate::lexical::{enum_variants, generic_end};
pub(super) struct Vocabulary(BTreeMap<String, BTreeSet<String>>);
impl Vocabulary {
    pub fn read(input: &Inputs, roots: &BTreeSet<String>) -> Self {
        let mut pairs = BTreeSet::new();
        for (path, source) in &input.files {
            if !path.ends_with(".rs")
                || !source.contains("enum")
                || !roots.iter().any(|root| source.contains(root))
            {
                continue;
            }
            for (owner, variant) in enum_variants(&tokens(source)) {
                if roots.contains(variant) && !roots.contains(owner) {
                    pairs.insert((owner.to_owned(), variant.to_owned()));
                }
            }
        }
        let owners: BTreeSet<_> = pairs.iter().map(|(owner, _)| owner.as_str()).collect();
        let mut shadowed = BTreeSet::new();
        for (path, source) in &input.files {
            if !path.ends_with(".rs") || !owners.iter().any(|owner| source.contains(owner)) {
                continue;
            }
            let tokens = tokens(source);
            for pair in tokens.windows(2) {
                // A conflicting type/module declaration or alias is ambiguous.
                // Keep the original conservative runtime finding in that case.
                // Same-named enums in different audiences still name values.
                if ["struct", "trait", "type", "union", "mod", "as"].contains(&pair[0].text)
                    && owners.contains(pair[1].text)
                {
                    shadowed.insert(pair[1].text.to_owned());
                }
            }
            for name in type_parameters(&tokens) {
                if owners.contains(name) {
                    shadowed.insert(name.to_owned());
                }
            }
        }
        let mut known: BTreeMap<String, BTreeSet<String>> = BTreeMap::new();
        for (owner, variant) in pairs {
            if !shadowed.contains(&owner) {
                known.entry(owner).or_default().insert(variant);
            }
        }
        Self(known)
    }
    pub fn variant(&self, t: &[Token<'_>], at: usize) -> bool {
        at >= 3
            && t[at - 1].text == ":"
            && t[at - 2].text == ":"
            && t.get(at + 1).is_none_or(|token| token.text != ":")
            && self
                .0
                .get(t[at - 3].text)
                .is_some_and(|variants| variants.contains(t[at].text))
    }
}

// Only declaration parameters shadow the enum type, not ordinary type arguments.
fn type_parameters<'a>(t: &[Token<'a>]) -> Vec<&'a str> {
    let mut names = Vec::new();
    for (i, token) in t.iter().enumerate() {
        let start = match token.text {
            "impl" => i + 1,
            "fn" | "struct" | "enum" | "trait" | "type" | "union"
                if t.get(i + 1)
                    .and_then(|token| token.text.chars().next())
                    .is_some_and(|first| {
                        first == '_' || first.is_ascii_alphabetic() || !first.is_ascii()
                    }) =>
            {
                i + 2
            }
            _ => continue,
        };
        if t.get(start).is_none_or(|token| token.text != "<") {
            continue;
        }
        let end = generic_end(t, start);
        let (mut at, mut first) = (start + 1, true);
        while at < end && at < t.len() {
            if t[at].text == "#" && t.get(at + 1).is_some_and(|token| token.text == "[") {
                at = close(t, at + 1) + 1;
                continue;
            }
            if first && t[at].text != "," {
                // Lifetime and const parameters occupy different namespaces.
                if !["'", "const"].contains(&t[at].text) {
                    names.push(t[at].text);
                }
                first = false;
            }
            if t[at].text == "," {
                first = true;
            }
            at = if t[at].text == "<" {
                generic_end(t, at)
            } else {
                close(t, at)
            } + 1;
        }
    }
    names
}
#[cfg(test)]
mod tests;
