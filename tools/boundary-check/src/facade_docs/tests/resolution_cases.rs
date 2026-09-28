use super::{site_of, Fixture};
use crate::facade_docs::observation::NameStatus;

const OWNER_LIB: &str = "\
pub mod facade;
mod inner;
pub use inner::deep::Original as Renamed;
";
const OWNER_INNER: &str = "\
pub(crate) mod deep;
pub use self::deep::*;
";
const OWNER_DEEP: &str = "\
/// The original definition.
pub struct Original;

pub enum Choice {
    /// The documented variant.
    First,
    Second,
}

pub fn undocumented_function() {}
";
const OWNER_FACADE: &str = "\
pub mod surface {
    pub use crate::inner::*;
}
";
const CONSUMER_FACADE: &str = "\
pub use fixture_owner::facade::surface;
pub use fixture_owner::facade::surface::Choice::First as FirstChoice;
pub use fixture_owner::facade::surface::{undocumented_function, Choice};
pub use fixture_owner::Renamed as Public;
pub use serde::Serialize;
";

fn chain_fixture() -> Fixture {
    let mut fixture = Fixture::new();
    fixture.crate_with(
        "fixture-owner",
        &[],
        &[
            ("lib.rs", OWNER_LIB),
            ("inner.rs", OWNER_INNER),
            ("inner/deep.rs", OWNER_DEEP),
            ("facade.rs", OWNER_FACADE),
        ],
    );
    fixture.crate_with(
        "fixture-consumer",
        &["fixture-owner"],
        &[
            ("lib.rs", "pub mod facade;\n"),
            ("facade.rs", CONSUMER_FACADE),
        ],
    );
    fixture
}

#[test]
fn renames_globs_and_cross_crate_reexports_reach_the_definition() {
    let fixture = chain_fixture();
    let observation = fixture.observe(&[
        (
            "fixture-consumer",
            &[
                "Choice",
                "FirstChoice",
                "Public",
                "Serialize",
                "surface",
                "undocumented_function",
            ],
        ),
        ("fixture-consumer::surface", &["Choice", "Original"]),
    ]);
    let status = |package, name| observation.status(package, name).cloned();
    assert_eq!(
        status("fixture-consumer", "Public"),
        Some(NameStatus::Documented)
    );
    assert_eq!(
        status("fixture-consumer", "FirstChoice"),
        Some(NameStatus::Documented)
    );
    assert_eq!(
        status("fixture-consumer", "Serialize"),
        Some(NameStatus::OutsideWorkspace)
    );
    assert_eq!(
        status("fixture-consumer::surface", "Original"),
        Some(NameStatus::Documented)
    );
    assert_eq!(
        site_of(&observation, "fixture-consumer", "undocumented_function"),
        ["crates/fixture-owner/src/inner/deep.rs:10"]
    );
    assert_eq!(
        site_of(&observation, "fixture-consumer::surface", "Choice"),
        ["crates/fixture-owner/src/inner/deep.rs:4"]
    );
    assert_eq!(
        site_of(&observation, "fixture-consumer", "surface"),
        ["crates/fixture-owner/src/facade.rs:1"]
    );
}

#[test]
fn private_items_stay_out_of_cross_crate_globs() {
    let mut fixture = Fixture::new();
    fixture.crate_with(
        "fixture-owner",
        &[],
        &[
            ("lib.rs", "pub mod facade;\nmod hidden;\n"),
            (
                "hidden.rs",
                "/// Crate-private.\npub(crate) struct Hidden;\n",
            ),
            ("facade.rs", "pub use crate::hidden::*;\n"),
        ],
    );
    let observation = fixture.observe(&[("fixture-owner", &["Hidden"])]);
    assert_eq!(
        observation.status("fixture-owner", "Hidden"),
        Some(&NameStatus::Documented)
    );
    fixture.crate_with(
        "fixture-consumer",
        &["fixture-owner"],
        &[
            ("lib.rs", "pub mod facade;\n"),
            ("facade.rs", "pub use fixture_owner::facade::*;\n"),
        ],
    );
    let observation = fixture.observe(&[("fixture-consumer", &["Hidden"])]);
    assert!(matches!(
        observation.status("fixture-consumer", "Hidden"),
        Some(NameStatus::Unresolved(_))
    ));
}

const MACRO_LIB: &str = "\
pub mod facade;
mod generated;

/// Declares a documented macro.
#[macro_export]
macro_rules! declare_value {
    ($name:ident) => {
        pub struct $name;
    };
}
";
const MACRO_GENERATED: &str = "\
macro_rules! plain_basis {
    ($name:ident) => {
        pub struct $name(u8);
    };
}

macro_rules! documented_basis {
    ($name:ident) => {
        #[doc = \"A generated basis.\"]
        pub struct $name(u8);
    };
}

plain_basis!(BareBasis);

/// Documented at the invocation.
plain_basis!(InvocationBasis);

documented_basis!(TemplateBasis);

macro_rules! accessors {
    ($type:ty; $($name:ident),+) => {
        impl $type { $(pub const fn $name(self) {})+ }
    };
}

/// Declared beside an impl-only macro invocation.
pub struct Declared;
accessors!(Declared; first);
";
const MACRO_FACADE: &str = "\
pub use crate::declare_value;
pub use crate::generated::{BareBasis, Declared, InvocationBasis, TemplateBasis};
";

#[test]
fn macro_generated_items_accept_invocation_or_template_docs() {
    let mut fixture = Fixture::new();
    fixture.crate_with(
        "macro-fixture",
        &[],
        &[
            ("lib.rs", MACRO_LIB),
            ("generated.rs", MACRO_GENERATED),
            ("facade.rs", MACRO_FACADE),
        ],
    );
    let names: &[&str] = &[
        "BareBasis",
        "Declared",
        "InvocationBasis",
        "TemplateBasis",
        "declare_value",
    ];
    let observation = fixture.observe(&[("macro-fixture", names)]);
    for documented in [
        "Declared",
        "InvocationBasis",
        "TemplateBasis",
        "declare_value",
    ] {
        assert_eq!(
            observation.status("macro-fixture", documented),
            Some(&NameStatus::Documented),
            "{documented}"
        );
    }
    assert_eq!(
        site_of(&observation, "macro-fixture", "BareBasis"),
        ["crates/macro-fixture/src/generated.rs:14"]
    );
}

#[test]
fn module_docs_count_inner_and_declaration_comments() {
    let mut fixture = Fixture::new();
    fixture.crate_with(
        "module-fixture",
        &[],
        &[
            (
                "lib.rs",
                "pub mod facade;\nmod inner_doc;\n/// Outer.\nmod outer_doc;\nmod bare;\nmod r#async;\n",
            ),
            ("inner_doc.rs", "//! Inner.\n"),
            ("outer_doc.rs", ""),
            ("bare.rs", ""),
            ("async.rs", "/// A raw-keyword module's item.\npub struct Later;\n"),
            (
                "facade.rs",
                "pub use crate::r#async::Later;\npub use crate::{bare, inner_doc, outer_doc};\n",
            ),
        ],
    );
    let observation = fixture.observe(&[(
        "module-fixture",
        &["Later", "bare", "inner_doc", "outer_doc"],
    )]);
    for documented in ["Later", "inner_doc", "outer_doc"] {
        assert_eq!(
            observation.status("module-fixture", documented),
            Some(&NameStatus::Documented)
        );
    }
    assert_eq!(
        site_of(&observation, "module-fixture", "bare"),
        ["crates/module-fixture/src/lib.rs:5"]
    );
}
