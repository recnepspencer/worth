//! Flatten one `use` tree into the names it binds and the paths they name.

use syn::{ItemUse, UseTree};

/// The name a `use` binding introduces into its module.
#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) enum BoundName {
    Named(String),
    Glob,
}

/// One flattened binding: the source path and the name it binds.
///
/// A path that starts at the extern prelude (`::name`) keeps `"::"` as its
/// first segment.
#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct UseBinding {
    pub(crate) segments: Vec<String>,
    pub(crate) bound: BoundName,
}

pub(crate) fn use_bindings(item: &ItemUse) -> Vec<UseBinding> {
    let mut prefix = Vec::new();
    if item.leading_colon.is_some() {
        prefix.push("::".to_owned());
    }
    let mut bindings = Vec::new();
    flatten(&item.tree, &mut prefix, &mut bindings);
    bindings
}

fn flatten(tree: &UseTree, prefix: &mut Vec<String>, bindings: &mut Vec<UseBinding>) {
    match tree {
        UseTree::Path(path) => {
            prefix.push(path.ident.to_string());
            flatten(&path.tree, prefix, bindings);
            prefix.pop();
        }
        UseTree::Name(name) if name.ident == "self" => {
            if let Some(last) = prefix.last() {
                bindings.push(named(prefix.clone(), last.clone()));
            }
        }
        UseTree::Name(name) => {
            let mut segments = prefix.clone();
            segments.push(name.ident.to_string());
            bindings.push(named(segments, name.ident.to_string()));
        }
        UseTree::Rename(rename) => {
            let mut segments = prefix.clone();
            if rename.ident != "self" {
                segments.push(rename.ident.to_string());
            }
            bindings.push(named(segments, rename.rename.to_string()));
        }
        UseTree::Glob(_) => bindings.push(UseBinding {
            segments: prefix.clone(),
            bound: BoundName::Glob,
        }),
        UseTree::Group(group) => {
            for item in &group.items {
                flatten(item, prefix, bindings);
            }
        }
    }
}

fn named(segments: Vec<String>, name: String) -> UseBinding {
    UseBinding {
        segments,
        bound: BoundName::Named(name),
    }
}
