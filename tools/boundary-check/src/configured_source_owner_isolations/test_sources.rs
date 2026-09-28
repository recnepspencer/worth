//! Which owner sources are test-only. Only a `#[cfg(test)]` module
//! declaration makes a file a test, never its name, and a file stays
//! production while any production declaration reaches it, so a test-only
//! `#[path]` to a production file exempts nothing. Declarations inside inline
//! modules count, and an inline `#[cfg(test)]` module makes every file it
//! declares a test. A file whose parent module file lies outside the owner
//! roots is always production, since that parent may declare it.

use std::collections::BTreeSet;

#[cfg(test)]
mod tests;

pub(super) struct TestSources {
    /// The module stem, `dir/name`, of every production source.
    production: BTreeSet<String>,
}

struct Declaration {
    /// The stem of the declaring file.
    from: String,
    /// The stem of the declared file.
    declared: String,
    test: bool,
}

impl TestSources {
    pub(super) fn new(sources: &[(String, syn::File)]) -> Self {
        let stems = sources
            .iter()
            .map(|(path, _)| module_stem(&normalized(path)).to_owned())
            .collect::<BTreeSet<_>>();
        let mut declarations = Vec::new();
        for (path, file) in sources {
            let path = normalized(path);
            let (directory, name) = path.rsplit_once('/').unwrap_or(("", path.as_str()));
            let file_stem = name.trim_end_matches(".rs");
            let children = if matches!(file_stem, "mod" | "lib" | "main") {
                directory.to_owned()
            } else {
                format!("{directory}/{file_stem}")
            };
            let from = module_stem(&path);
            declare(
                &file.items,
                from,
                directory,
                &children,
                false,
                &mut declarations,
            );
        }
        // A file whose parent module file lies outside the owner roots may be
        // declared from there, so it is a production root whatever the owner
        // declares; a file its parent never declares counts as its child.
        let declared = declarations
            .iter()
            .map(|declaration| declaration.declared.clone())
            .collect::<BTreeSet<_>>();
        let mut production = BTreeSet::new();
        for stem in &stems {
            match parent_file(stem, &stems) {
                None => {
                    production.insert(stem.clone());
                }
                Some(parent) if !declared.contains(stem) => declarations.push(Declaration {
                    from: parent.to_owned(),
                    declared: stem.clone(),
                    test: false,
                }),
                Some(_) => {}
            }
        }
        loop {
            let reached = production.len();
            for declaration in &declarations {
                if !declaration.test && production.contains(&declaration.from) {
                    production.insert(declaration.declared.clone());
                }
            }
            if production.len() == reached {
                break;
            }
        }
        Self { production }
    }

    /// Whether the owner source at `path` is test-only.
    pub(super) fn contains(&self, path: &str) -> bool {
        !self.production.contains(module_stem(&normalized(path)))
    }
}

fn declare(
    items: &[syn::Item],
    from: &str,
    path_base: &str,
    children: &str,
    test: bool,
    declarations: &mut Vec<Declaration>,
) {
    for item in items {
        let syn::Item::Mod(module) = item else {
            continue;
        };
        let test = test || is_test_only(&module.attrs);
        match &module.content {
            Some((_, items)) => {
                let nested = format!("{children}/{}", module.ident);
                declare(items, from, &nested, &nested, test, declarations);
            }
            None => {
                let declared = match path_attribute(&module.attrs) {
                    Some(relative) => format!("{path_base}/{relative}"),
                    None => format!("{children}/{}", module.ident),
                };
                declarations.push(Declaration {
                    from: from.to_owned(),
                    declared: module_stem(&normalized(&declared)).to_owned(),
                    test,
                });
            }
        }
    }
}

/// The nearest owner file above `stem`, which declares it or the inline
/// module holding it: `dir.rs`, `dir/mod.rs`, `dir/lib.rs` or `dir/main.rs`
/// for the closest such `dir`. A crate root has none.
fn parent_file<'a>(stem: &str, stems: &'a BTreeSet<String>) -> Option<&'a str> {
    let (mut above, name) = stem.rsplit_once('/')?;
    if matches!(name, "lib" | "main") {
        return None;
    }
    loop {
        let found = [
            above.to_owned(),
            format!("{above}/lib"),
            format!("{above}/main"),
        ]
        .iter()
        .find_map(|parent| stems.get(parent));
        if let Some(parent) = found {
            return Some(parent);
        }
        above = above.rsplit_once('/')?.0;
    }
}

/// The path with forward slashes and every `.` and `..` segment resolved.
fn normalized(path: &str) -> String {
    let mut segments: Vec<&str> = Vec::new();
    for segment in path.split(['/', '\\']) {
        match segment {
            "." => {}
            ".." if segments.last().is_some_and(|last| *last != "..") => {
                segments.pop();
            }
            _ => segments.push(segment),
        }
    }
    segments.join("/")
}

/// `dir/name.rs` and `dir/name/mod.rs` both name the module `dir/name`.
fn module_stem(path: &str) -> &str {
    let stem = path.trim_end_matches(".rs");
    stem.strip_suffix("/mod").unwrap_or(stem)
}

fn path_attribute(attributes: &[syn::Attribute]) -> Option<String> {
    attributes
        .iter()
        .find_map(|attribute| match &attribute.meta {
            syn::Meta::NameValue(syn::MetaNameValue {
                path,
                value:
                    syn::Expr::Lit(syn::ExprLit {
                        lit: syn::Lit::Str(value),
                        ..
                    }),
                ..
            }) if path.is_ident("path") => Some(value.value()),
            _ => None,
        })
}

pub(super) fn is_test_only(attributes: &[syn::Attribute]) -> bool {
    attributes.iter().any(|attribute| {
        attribute.path().is_ident("cfg")
            && attribute
                .parse_args::<syn::Ident>()
                .is_ok_and(|predicate| predicate == "test")
    })
}
