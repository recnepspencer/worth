use super::document::{FacadeDocument, FacadeRow, SCHEMA_VERSION};
use super::facade_reexport_validation::validate_exact_public_reexport;
use super::facade_visibility::validate_facade_only_surface;
use crate::manifest_types::Road1Package;
use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::path::{Path, PathBuf};
use syn::{Item, UseTree, Visibility};

pub(crate) struct ObservedFacadeExports {
    by_package: BTreeMap<String, BTreeSet<String>>,
}

pub(crate) struct ConfiguredFacadeSurface {
    pub(crate) label: String,
    pub(crate) path: PathBuf,
    pub(crate) namespace: Option<String>,
    pub(crate) reexport: Option<String>,
    pub(crate) owner_path: Option<PathBuf>,
}

impl ObservedFacadeExports {
    pub(super) fn from_document(document: &FacadeDocument) -> Self {
        Self {
            by_package: document
                .facades
                .iter()
                .map(|row| (row.package.clone(), row.exports.iter().cloned().collect()))
                .collect(),
        }
    }

    pub(crate) fn names_for<'a>(
        &'a self,
        packages: impl Iterator<Item = &'a str>,
    ) -> BTreeSet<String> {
        packages
            .filter_map(|package| self.by_package.get(package))
            .flatten()
            .cloned()
            .collect()
    }
}

pub(super) fn observe_facade_document(
    packages: &[Road1Package],
    configured_surfaces: &[ConfiguredFacadeSurface],
) -> Result<FacadeDocument, String> {
    let mut facades = packages
        .iter()
        .map(|package| {
            let manifest = PathBuf::from(&package.manifest_path);
            let facade = validate_facade_only_surface(&manifest)?;
            Ok(FacadeRow {
                package: package.name.clone(),
                exports: extract_exports(&facade)?,
            })
        })
        .collect::<Result<Vec<_>, String>>()?;
    for surface in configured_surfaces {
        if facades.iter().any(|row| row.package == surface.label) {
            return Err(format!(
                "configured facade surface label is duplicated: {}",
                surface.label
            ));
        }
        if let Some(reexport) = &surface.reexport {
            validate_exact_public_reexport(&surface.path, reexport)?;
        }
        let observed_path = surface.owner_path.as_ref().unwrap_or(&surface.path);
        facades.push(FacadeRow {
            package: surface.label.clone(),
            exports: extract_namespace_exports(observed_path, surface.namespace.as_deref())?,
        });
    }
    facades.sort_by(|left, right| left.package.cmp(&right.package));
    Ok(FacadeDocument {
        schema_version: SCHEMA_VERSION,
        facades,
    })
}

fn extract_namespace_exports(path: &Path, namespace: Option<&str>) -> Result<Vec<String>, String> {
    let text =
        fs::read_to_string(path).map_err(|error| format!("read {}: {error}", path.display()))?;
    let syntax = crate::source_syntax::parse_file(&text)
        .map_err(|error| format!("parse {}: {error}", path.display()))?;
    let mut exports = BTreeSet::new();
    let items = match namespace {
        Some(namespace) => selected_namespace_items(&syntax.items, namespace, path)?,
        None => &syntax.items,
    };
    collect_namespace_items(items, "", &mut exports, path)?;
    Ok(exports.into_iter().collect())
}

fn selected_namespace_items<'a>(
    items: &'a [Item],
    namespace: &str,
    path: &Path,
) -> Result<&'a [Item], String> {
    let module = items
        .iter()
        .filter_map(|item| match item {
            Item::Mod(module) if module.ident == namespace => Some(module),
            _ => None,
        })
        .next()
        .ok_or_else(|| {
            format!(
                "configured facade namespace `{namespace}` is absent from {}",
                path.display()
            )
        })?;
    if !matches!(module.vis, Visibility::Public(_)) {
        return Err(format!(
            "configured facade namespace `{namespace}` is not public in {}",
            path.display()
        ));
    }
    module
        .content
        .as_ref()
        .map(|(_, items)| items.as_slice())
        .ok_or_else(|| {
            format!(
                "configured facade namespace `{namespace}` must be inline in {}",
                path.display()
            )
        })
}

fn collect_namespace_items(
    items: &[Item],
    prefix: &str,
    exports: &mut BTreeSet<String>,
    path: &Path,
) -> Result<(), String> {
    for item in items {
        match item {
            Item::Use(item_use) if matches!(item_use.vis, Visibility::Public(_)) => {
                let mut local = BTreeSet::new();
                collect(&item_use.tree, None, &mut local, path)?;
                exports.extend(local.into_iter().map(|name| qualify(prefix, &name)));
            }
            Item::Mod(module) if matches!(module.vis, Visibility::Public(_)) => {
                let name = qualify(prefix, &module.ident.to_string());
                exports.insert(name.clone());
                let Some((_, nested)) = &module.content else {
                    return Err(format!(
                        "configured nested facade modules must be inline in {}",
                        path.display()
                    ));
                };
                collect_namespace_items(nested, &name, exports, path)?;
            }
            _ => {
                return Err(format!(
                    "configured facade surface must contain only public re-exports or inline namespaces in {}",
                    path.display()
                ));
            }
        }
    }
    Ok(())
}

fn qualify(prefix: &str, name: &str) -> String {
    if prefix.is_empty() {
        name.to_owned()
    } else {
        format!("{prefix}::{name}")
    }
}

fn extract_exports(path: &Path) -> Result<Vec<String>, String> {
    let text =
        fs::read_to_string(path).map_err(|e| format!("read facade {}: {e}", path.display()))?;
    let syntax = crate::source_syntax::parse_file(&text)
        .map_err(|e| format!("parse facade {}: {e}", path.display()))?;
    let mut exports = BTreeSet::new();
    for item in syntax.items {
        match item {
            Item::Use(item_use) if matches!(item_use.vis, Visibility::Public(_)) => {
                if item_use.attrs.iter().any(|attribute| {
                    attribute.path().is_ident("cfg") || attribute.path().is_ident("cfg_attr")
                }) {
                    return Err(format!(
                        "conditional or attributed facade exports cannot form an exact compiled surface in {}",
                        path.display()
                    ));
                }
                collect(&item_use.tree, None, &mut exports, path)?;
            }
            _ => {
                return Err(format!(
                    "facade.rs must aggregate public exports only in {}",
                    path.display()
                ));
            }
        }
    }
    Ok(exports.into_iter().collect())
}

fn collect(
    tree: &UseTree,
    preceding_segment: Option<&str>,
    exports: &mut BTreeSet<String>,
    path: &Path,
) -> Result<(), String> {
    match tree {
        UseTree::Path(value) => {
            let segment = value.ident.to_string();
            collect(&value.tree, Some(&segment), exports, path)
        }
        UseTree::Name(value) => {
            let name = if value.ident == "self" {
                preceding_segment
                    .ok_or_else(|| {
                        format!("self export has no preceding path in {}", path.display())
                    })?
                    .to_owned()
            } else {
                value.ident.to_string()
            };
            exports.insert(name);
            Ok(())
        }
        UseTree::Rename(value) => {
            exports.insert(value.rename.to_string());
            Ok(())
        }
        UseTree::Group(value) => value
            .items
            .iter()
            .try_for_each(|item| collect(item, preceding_segment, exports, path)),
        UseTree::Glob(_) => Err(format!(
            "glob export cannot form exact facade set in {}",
            path.display()
        )),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn direct_grouped_and_renamed_exports_are_exact() {
        let syntax =
            crate::source_syntax::parse_file("pub use x::{A, B as C};\nuse x::Private;").unwrap();
        let mut exports = BTreeSet::new();
        for item in syntax.items {
            if let Item::Use(item) = item {
                if matches!(item.vis, Visibility::Public(_)) {
                    collect(&item.tree, None, &mut exports, Path::new("facade.rs")).unwrap();
                }
            }
        }
        assert_eq!(exports.into_iter().collect::<Vec<_>>(), ["A", "C"]);
    }

    #[test]
    fn glob_is_denied() {
        let tree: UseTree = syn::parse_str("x::*").unwrap();
        assert!(collect(&tree, None, &mut BTreeSet::new(), Path::new("facade.rs")).is_err());
    }

    #[test]
    fn self_exports_use_the_preceding_public_path_segment() {
        let tree: UseTree = syn::parse_str("outer::first::{self, Item}").unwrap();
        let mut exports = BTreeSet::new();
        collect(&tree, None, &mut exports, Path::new("facade.rs")).unwrap();
        assert_eq!(exports.into_iter().collect::<Vec<_>>(), ["Item", "first"]);
    }

    #[test]
    fn non_reexport_item_is_denied() {
        let path = temporary_facade("surface", "pub struct FirstSurface;\n");
        let error = extract_exports(&path).unwrap_err();
        fs::remove_file(&path).unwrap();
        assert!(error.contains("facade.rs must aggregate public exports only"));
    }

    #[test]
    fn attributed_public_reexport_is_denied() {
        let path = temporary_facade("attrs", "#[cfg(feature = \"wide\")]\npub use x::Wide;\n");
        let error = extract_exports(&path).unwrap_err();
        fs::remove_file(&path).unwrap();
        assert!(error.contains("cannot form an exact compiled surface"));
    }

    #[test]
    fn configured_namespace_surface_records_qualified_nested_exports() {
        let path = temporary_facade(
            "namespace",
            "pub use x::Root;\npub mod nested { pub use y::{First, Second}; }\n",
        );
        let exports = extract_namespace_exports(&path, None).unwrap();
        fs::remove_file(&path).unwrap();
        assert_eq!(
            exports,
            ["Root", "nested", "nested::First", "nested::Second"]
        );
    }

    #[test]
    fn configured_namespace_surface_rejects_behavior_items() {
        let path = temporary_facade("namespace-behavior", "pub fn bypass() {}\n");
        let error = extract_namespace_exports(&path, None).unwrap_err();
        fs::remove_file(&path).unwrap();
        assert!(error.contains("only public re-exports or inline namespaces"));
    }

    #[test]
    fn selected_namespace_snapshot_detects_nested_api_mutation() {
        let path = temporary_facade(
            "namespace-mutation",
            "pub mod primary_graph { pub use x::First; }\npub mod unrelated { pub use y::Ignored; }\n",
        );
        let before = extract_namespace_exports(&path, Some("primary_graph")).unwrap();
        fs::write(
            &path,
            "pub mod primary_graph { pub use x::{First, Second}; }\npub mod unrelated { pub use y::Ignored; }\n",
        )
        .unwrap();
        let after = extract_namespace_exports(&path, Some("primary_graph")).unwrap();
        fs::remove_file(&path).unwrap();
        assert_eq!(before, ["First"]);
        assert_eq!(after, ["First", "Second"]);
        assert_ne!(before, after);
    }

    fn temporary_facade(label: &str, contents: &str) -> PathBuf {
        let id = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let path = std::env::temp_dir().join(format!("boundary-check-facade-{label}-{id}.rs"));
        fs::write(&path, contents).unwrap();
        path
    }
}
