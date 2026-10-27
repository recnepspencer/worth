//! Rustc source selection and conditional fallback, independent of file contents.
use super::resolve_child_sources;
use std::{fs, path::PathBuf};
struct Sources(PathBuf);
impl Sources {
    fn new() -> Self {
        let root = worth_harness::tooling::fixture_path::unique_path("module-source-selection");
        fs::create_dir_all(&root).unwrap();
        for file in ["body.rs", "selected.rs", "alternate.rs"] {
            fs::write(root.join(file), "pub fn value() {}\n").unwrap();
        }
        Self(root)
    }
    fn resolve(&self, item: syn::ItemMod) -> Result<Vec<PathBuf>, String> {
        resolve_child_sources(&self.0, &self.0, "body", &item.attrs)
    }
}
impl Drop for Sources {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.0);
    }
}
#[test]
fn direct_path_excludes_the_unselected_conventional_source() {
    let sources = Sources::new();
    assert_eq!(
        sources
            .resolve(syn::parse_quote!(
                #[path = "selected.rs"]
                mod body;
            ))
            .unwrap(),
        vec![sources.0.join("selected.rs")]
    );
}
#[test]
fn conditional_paths_keep_every_selectable_source_and_conventional_fallback() {
    let sources = Sources::new();
    let actual = sources
        .resolve(syn::parse_quote!(
            #[cfg_attr(test, path = "selected.rs")]
            #[cfg_attr(not(test), path = "alternate.rs")]
            mod body;
        ))
        .unwrap();
    assert_eq!(
        actual
            .into_iter()
            .collect::<std::collections::BTreeSet<_>>(),
        ["selected.rs", "alternate.rs", "body.rs"]
            .into_iter()
            .map(|name| sources.0.join(name))
            .collect()
    );
}
#[test]
fn malformed_or_missing_direct_path_cannot_fall_back_to_a_real_decoy() {
    let sources = Sources::new();
    assert!(sources
        .resolve(syn::parse_quote!(
            #[path = 7]
            mod body;
        ))
        .is_err());
    assert!(sources
        .resolve(syn::parse_quote!(
            #[path = 7]
            #[cfg_attr(test, path = "selected.rs")]
            mod body;
        ))
        .is_err());
    assert!(sources
        .resolve(syn::parse_quote!(
            #[path = "absent.rs"]
            mod body;
        ))
        .is_err());
}
