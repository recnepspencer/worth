//! Compiler dependency discovery and semantic source inputs. Discovery is
//! performed by rustc, not a hand-maintained module or Cargo metadata walk.

use std::{
    collections::{BTreeMap, BTreeSet},
    fs,
    path::{Path, PathBuf},
};

use proc_macro2::{TokenStream, TokenTree};
use serde_json::{json, Value};
use syn::parse::Parser;

use crate::{compiler_context, fingerprint};

pub(crate) struct Discovered {
    pub(crate) files: Vec<PathBuf>,
    pub(crate) environment: BTreeMap<String, String>,
}

pub(crate) fn dep_info(path: &Path) -> Result<Discovered, String> {
    let text = fs::read_to_string(path).map_err(|error| error.to_string())?;
    let mut files = BTreeSet::new();
    let mut environment = BTreeMap::new();
    for line in text.lines() {
        if let Some(entry) = line.strip_prefix("# env-dep:") {
            let (name, value) = entry.split_once('=').unwrap_or((entry, "<unset>"));
            if name == crate::cargo_build::REOBSERVE {
                continue;
            }
            environment.insert(name.into(), value.into());
        } else if let Some((_, dependencies)) = line.split_once(": ") {
            for dependency in make_words(dependencies) {
                let path = fs::canonicalize(&dependency)
                    .map_err(|error| format!("compiler input {dependency}: {error}"))?;
                files.insert(path);
            }
        }
    }
    if files.is_empty() {
        return Err("compiler returned no dependency inputs".into());
    }
    Ok(Discovered {
        files: files.into_iter().collect(),
        environment,
    })
}

fn make_words(line: &str) -> Vec<String> {
    let mut words = Vec::new();
    let mut word = String::new();
    let mut chars = line.chars().peekable();
    while let Some(character) = chars.next() {
        if character == '\\'
            && chars.peek().is_some_and(|next| {
                matches!(next, ' ' | '#' | '\t') || (!cfg!(windows) && *next == '\\')
            })
        {
            word.push(chars.next().unwrap());
        } else if character.is_whitespace() {
            if !word.is_empty() {
                words.push(std::mem::take(&mut word));
            }
        } else {
            word.push(character);
        }
    }
    if !word.is_empty() {
        words.push(word);
    }
    words
}

pub(crate) fn records(
    discovered: &Discovered,
    session: &Value,
    test: bool,
) -> Result<Vec<Value>, String> {
    let raw = raw_includes(discovered)?;
    discovered.files.iter().map(|path| {
        let content = fs::read(path).map_err(|error| error.to_string())?;
        let raw = raw.contains(path);
        let semantic = if raw { fingerprint::bytes(&content) } else { content_digest(path, &content, test)? };
        Ok(json!({
            "path": path, "identity": compiler_context::normalize_path(&path.to_string_lossy(), session),
            "digest": semantic, "test": test, "raw": raw,
        }))
    }).collect()
}

pub(crate) fn environment(discovered: &Discovered, session: &Value) -> Result<Value, String> {
    let mut path_only = BTreeSet::new();
    let mut semantic = BTreeSet::new();
    let mut unknown = false;
    for path in &discovered.files {
        if path.extension().is_some_and(|extension| extension == "rs")
            && let Ok(tokens) = fs::read_to_string(path)
                .map_err(|error| error.to_string())?
                .parse::<TokenStream>()
        {
            environment_uses(tokens, false, &mut path_only, &mut semantic, &mut unknown);
        }
    }
    Ok(json!(discovered
        .environment
        .keys()
        .map(|name| {
            // Dep-info escapes its text representation (notably Windows
            // backslashes). The input is the actual environment passed to
            // rustc, selected by the compiler-discovered dependency name.
            let actual = std::env::var_os(name);
            let value = actual.as_ref().map(|value| {
                if !unknown && path_only.contains(name) && !semantic.contains(name) {
                    compiler_context::normalize_path(&value.to_string_lossy(), session).into_bytes()
                } else {
                    value.as_encoded_bytes().to_vec()
                }
            });
            (
                name.clone(),
                fingerprint::value("worth.compiler.environment.v1", &json!(value)),
            )
        })
        .collect::<BTreeMap<_, _>>()))
}

fn environment_uses(
    tokens: TokenStream,
    included: bool,
    paths: &mut BTreeSet<String>,
    semantic: &mut BTreeSet<String>,
    unknown: &mut bool,
) {
    let tokens: Vec<_> = tokens.into_iter().collect();
    let mut index = 0;
    while index < tokens.len() {
        if let (
            Some(TokenTree::Ident(name)),
            Some(TokenTree::Punct(mark)),
            Some(TokenTree::Group(group)),
        ) = (
            tokens.get(index),
            tokens.get(index + 1),
            tokens.get(index + 2),
        ) && mark.as_char() == '!'
        {
            let name = name.to_string();
            if matches!(name.as_str(), "env" | "option_env") {
                let values =
                    syn::punctuated::Punctuated::<syn::Expr, syn::Token![,]>::parse_terminated
                        .parse2(group.stream());
                if let Ok(values) = values {
                    if let Some(syn::Expr::Lit(syn::ExprLit {
                        lit: syn::Lit::Str(name),
                        ..
                    })) = values.first()
                    {
                        if included {
                            paths.insert(name.value());
                        } else {
                            semantic.insert(name.value());
                        }
                    } else {
                        *unknown = true;
                    }
                } else {
                    *unknown = true;
                }
            } else {
                environment_uses(
                    group.stream(),
                    included
                        || matches!(name.as_str(), "include" | "include_str" | "include_bytes"),
                    paths,
                    semantic,
                    unknown,
                );
            }
            index += 3;
            continue;
        }
        if let TokenTree::Group(group) = &tokens[index] {
            environment_uses(group.stream(), included, paths, semantic, unknown);
        }
        index += 1;
    }
}

fn raw_includes(discovered: &Discovered) -> Result<BTreeSet<PathBuf>, String> {
    let mut raw = BTreeSet::new();
    for path in &discovered.files {
        if path.extension().is_some_and(|extension| extension == "rs")
            && let Ok(tokens) = fs::read_to_string(path)
                .map_err(|error| error.to_string())?
                .parse::<TokenStream>()
        {
            find_raw_includes(tokens, path.parent().unwrap(), &mut raw)?;
        }
    }
    Ok(raw)
}

fn find_raw_includes(
    tokens: TokenStream,
    directory: &Path,
    raw: &mut BTreeSet<PathBuf>,
) -> Result<(), String> {
    let tokens: Vec<_> = tokens.into_iter().collect();
    for (index, token) in tokens.iter().enumerate() {
        if let TokenTree::Ident(name) = token
            && matches!(name.to_string().as_str(), "include_str" | "include_bytes")
            && let (Some(TokenTree::Punct(mark)), Some(TokenTree::Group(group))) =
                (tokens.get(index + 1), tokens.get(index + 2))
            && mark.as_char() == '!'
        {
            let literal = syn::parse2::<syn::LitStr>(group.stream())
                .map_err(|_| "computed include_str/include_bytes path requires qualification")?;
            raw.insert(
                fs::canonicalize(directory.join(literal.value()))
                    .map_err(|error| error.to_string())?,
            );
        }
        if let TokenTree::Group(group) = token {
            find_raw_includes(group.stream(), directory, raw)?;
        }
    }
    Ok(())
}

mod native_source;
mod source;
pub(crate) use source::content_digest;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn an_environment_value_with_a_message_is_still_semantic_outside_includes() {
        let tokens =
            r#"include!(env!("OUT_DIR")); const PATH: &str = env!("OUT_DIR", "required");"#
                .parse()
                .unwrap();
        let (mut paths, mut semantic, mut unknown) = (BTreeSet::new(), BTreeSet::new(), false);
        environment_uses(tokens, false, &mut paths, &mut semantic, &mut unknown);
        assert!(paths.contains("OUT_DIR"));
        assert!(semantic.contains("OUT_DIR"));
        assert!(!unknown);
    }
}
