//! Negative knowledge is not inferred from a failed rustdoc lookup.
use super::CrateIndex;
use super::{export_knowledge, Knowledge, SelectorKind};
fn enum_definition(
    root: &std::path::Path,
    manifest: &str,
    item: &str,
) -> Result<syn::ItemEnum, String> {
    super::enum_definition(root, manifest, item, &CrateIndex::new(root))
}
use std::path::PathBuf;
struct Fixture(PathBuf);
impl Fixture {
    fn new(source: &str) -> Self {
        let root = worth_harness::tooling::fixture_path::unique_path("facade-knowledge");
        std::fs::create_dir_all(root.join("crates/example/src")).unwrap();
        std::fs::write(
            root.join("crates/example/Cargo.toml"),
            "[package]\nname='example'\nversion='0.1.0'\n",
        )
        .unwrap();
        std::fs::write(root.join("crates/example/src/lib.rs"), source).unwrap();
        Self(root)
    }
    fn query(&self, container: &str, name: &str, kind: SelectorKind) -> Knowledge {
        export_knowledge(&self.0, "crates/example/Cargo.toml", container, name, kind)
    }
}
impl Drop for Fixture {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.0);
    }
}
#[test]
fn closed_absence_and_newly_present_export_are_distinct() {
    let fixture = Fixture::new("pub mod facade { pub enum Choice { First } }");
    assert_eq!(
        fixture.query("facade", "Missing", SelectorKind::ModuleExport),
        Knowledge::ProvenAbsent
    );
    assert_eq!(
        fixture.query("facade", "Choice", SelectorKind::ModuleExport),
        Knowledge::Present
    );
    assert_eq!(
        fixture.query("facade::Choice", "Second", SelectorKind::EnumVariant),
        Knowledge::ProvenAbsent
    );
    assert_eq!(
        fixture.query("facade::Choice", "First", SelectorKind::EnumVariant),
        Knowledge::Present
    );
    std::fs::write(
        fixture.0.join("crates/example/src/lib.rs"),
        "pub mod facade { pub struct Missing; }",
    )
    .unwrap();
    assert_eq!(
        fixture.query("facade", "Missing", SelectorKind::ModuleExport),
        Knowledge::Present
    );
}
#[test]
fn uncertain_exports_and_missing_sources_are_not_absence() {
    for source in [
        "pub mod facade { #[cfg(feature=\"unknown\")] pub struct Maybe; }",
        "pub mod facade { opaque!(); }",
        "mod facade {}",
        "mod owner { pub mod facade {} } #[cfg(feature=\"unknown\")] pub use owner::facade;",
        "pub mod facade { #[generate_exports] pub struct Generated; }",
        "pub mod facade { pub use external::*; }",
        "pub mod facade { pub use crate::facade::*; }",
        "pub mod facade;",
        "pub mod facade {",
    ] {
        let fixture = Fixture::new(source);
        assert!(
            matches!(
                fixture.query("facade", "Missing", SelectorKind::ModuleExport),
                Knowledge::Unsupported(_)
            ),
            "{source}"
        );
    }
    let fixture = Fixture::new("pub mod facade {}");
    assert!(matches!(
        fixture.query("absent_module", "Missing", SelectorKind::ModuleExport),
        Knowledge::Unsupported(_)
    ));
}
#[test]
fn out_of_file_and_macro_members_have_no_false_negative_proof() {
    let fixture = Fixture::new("pub mod facade { pub struct Thing; } mod implementation;");
    std::fs::write(
        fixture.0.join("crates/example/src/implementation.rs"),
        "impl crate::facade::Thing { pub fn exists() {} }",
    )
    .unwrap();
    assert!(matches!(
        fixture.query("facade::Thing", "exists", SelectorKind::AssociatedMember),
        Knowledge::Unsupported(_)
    ));
    let fixture = Fixture::new("pub mod facade { pub struct Thing; impl Thing { members!(); } }");
    assert!(matches!(
        fixture.query("facade::Thing", "exists", SelectorKind::AssociatedMember),
        Knowledge::Unsupported(_)
    ));
}
#[test]
fn facts_follow_real_reexports_but_not_attributed_macro_shapes() {
    let fixture = Fixture::new("mod owner { pub enum Choice { First, Second(u64) } } pub mod facade { pub use crate::owner::Choice; }");
    let item = enum_definition(&fixture.0, "crates/example/Cargo.toml", "facade::Choice").unwrap();
    assert_eq!(item.variants.len(), 2);
    let fixture = Fixture::new("pub mod facade { make!(Choice); }");
    assert!(enum_definition(&fixture.0, "crates/example/Cargo.toml", "facade::Choice").is_err());
}

#[test]
fn conditional_reexport_does_not_become_a_current_fact_route() {
    let fixture = Fixture::new("mod owner { pub enum Choice { First } } pub mod facade { #[cfg(feature=\"unknown\")] pub use crate::owner::Choice; }");
    assert!(enum_definition(&fixture.0, "crates/example/Cargo.toml", "facade::Choice").is_err());
}

#[test]
fn private_containers_and_imported_routes_are_not_closed_audience_knowledge() {
    let fixture = Fixture::new("pub mod facade { mod private {} }");
    assert!(matches!(
        fixture.query("facade::private", "Missing", SelectorKind::ModuleExport),
        Knowledge::Unsupported(_)
    ));
    let fixture = Fixture::new(
        "mod owner { pub mod exposed {} } pub mod facade { pub use crate::owner::exposed; }",
    );
    assert!(matches!(
        fixture.query("facade::exposed", "Missing", SelectorKind::ModuleExport),
        Knowledge::Unsupported(_)
    ));
}

#[test]
fn fact_routes_require_public_visibility_and_only_exact_benign_metadata() {
    for source in [
        "mod facade { pub enum Choice { First } }",
        "#[transform] pub mod facade { pub enum Choice { First } }",
        "#[cfg_attr(doc, cfg(any()))] pub mod facade { pub enum Choice { First } }",
        "#[cfg_attr(doc, recursion_limit=\"256\", cfg(any()))] pub mod facade { pub enum Choice { First } }",
    ] {
        let fixture = Fixture::new(source);
        assert!(enum_definition(&fixture.0, "crates/example/Cargo.toml", "facade::Choice").is_err(), "{source}");
    }
    let fixture = Fixture::new("#![cfg_attr(not(test), deny(clippy::iter_over_hash_type, clippy::disallowed_methods))] #![cfg_attr(test, allow(clippy::disallowed_methods))] #![cfg_attr(doc, recursion_limit=\"256\")] mod implementation { pub enum Choice { First } } pub mod facade { pub use crate::implementation::Choice; }");
    assert!(enum_definition(&fixture.0, "crates/example/Cargo.toml", "facade::Choice").is_ok());
}

#[test]
fn malformed_negative_identifiers_do_not_become_absence() {
    let fixture = Fixture::new("pub mod facade {}");
    for name in [
        "",
        "not a name",
        "Missing::Member",
        "123",
        "Missing()",
        "!Missing",
    ] {
        assert!(
            matches!(
                fixture.query("facade", name, SelectorKind::ModuleExport),
                Knowledge::Unsupported(_)
            ),
            "{name}"
        );
    }
}

#[test]
fn payload_names_follow_actual_public_aliases_and_refuse_private_shapes() {
    for (source, expected) in [
        ("mod owner { pub struct Payload; pub enum Choice { One(Payload) } } pub mod facade { pub use crate::owner::{Choice, Payload}; }", Some("Payload")),
        ("mod owner { pub struct Payload; pub enum Choice { One(Payload) } } pub mod facade { pub use crate::owner::Choice; pub use crate::owner::Payload as Visible; }", Some("Visible")),
        ("mod owner { struct Payload; pub enum Choice { One(Payload) } } pub mod facade { pub use crate::owner::Choice; }", None),
        ("#[transform] mod owner { pub struct Payload; pub enum Choice { One(Payload) } } pub mod facade { pub use crate::owner::{Choice, Payload}; }", None),
    ] {
        let fixture = Fixture::new(source);
        let crates = CrateIndex::new(&fixture.0);
        let result = super::enum_definition(&fixture.0, "crates/example/Cargo.toml", "facade::Choice", &crates)
            .and_then(|item| {
                let field = item.variants[0].fields.iter().next().unwrap();
                super::payload_name(&fixture.0, "crates/example/Cargo.toml", "facade::Choice", &item, &field.ty, &crates)
            });
        match expected { Some(name) => assert_eq!(result.unwrap(), name), None => assert!(result.is_err(), "{source}") }
    }
}
