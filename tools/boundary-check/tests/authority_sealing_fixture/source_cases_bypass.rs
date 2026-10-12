//! Bypass specimens: aliases, macro/attr fences, and external re-export targets.

/// Entry crate: public re-export of an external path dependency item.
pub fn entry_reexport_external_admit() -> &'static str {
    r#"
pub use worth_schema_external::admit;
"#
}

/// Entry crate: renamed public re-export of external item.
pub fn entry_reexport_external_admit_renamed() -> &'static str {
    r#"
pub use worth_schema_external::admit as public_admit;
"#
}

/// Entry crate: group public re-export of external items.
pub fn entry_reexport_external_group() -> &'static str {
    r#"
pub use worth_schema_external::{admit, seed};
"#
}

/// External package: concrete public ceremony (legal re-export target).
pub fn external_legal_concrete_admit() -> &'static str {
    r#"
pub struct EntryAdmission {
    _value_gate: (),
}

pub struct AuthorityWitnessPlaceholder<A> {
    _marker: core::marker::PhantomData<A>,
}

pub fn admit(_authority: AuthorityWitnessPlaceholder<EntryAdmission>) {}

pub fn seed() {}
"#
}

/// External package: generic AuthorityMarker ceremony (hostile re-export target).
pub fn external_hostile_generic_admit() -> &'static str {
    r#"
pub trait AuthorityMarker: 'static {}

pub fn admit<A: AuthorityMarker>(_authority: A) {}

pub fn seed() {}
"#
}

/// External package: generic ceremony bound via a local rename of AuthorityMarker.
pub fn external_hostile_aliased_marker_admit() -> &'static str {
    r#"
pub trait AuthorityMarker: 'static {}

use AuthorityMarker as AuthMark;

pub fn admit<A: AuthMark>(_authority: A) {}

pub fn seed() {}
"#
}

/// Hostile: #[macro_export] macro_rules body containing AuthorityMarker ceremony.
pub fn hostile_macro_export_generic_ceremony() -> &'static str {
    r#"
#[macro_export]
macro_rules! open_ceremony {
    () => {
        pub fn admit<A: AuthorityMarker>(_authority: A) {}
    };
}

pub fn seed() {}
"#
}

/// Legal: private (non-exported) macro may mention AuthorityMarker; not public API.
pub fn legal_private_macro_rules_with_marker() -> &'static str {
    r#"
macro_rules! internal_open {
    () => {
        fn admit<A: AuthorityMarker>(_authority: A) {}
    };
}

pub fn seed() {
    // macro remains unexported and uninvoked in item position
}
"#
}

/// Hostile: opaque custom attribute on a public signature-bearing item.
pub fn hostile_opaque_attribute_on_public_fn() -> &'static str {
    r#"
#[ceremony_factory]
pub fn admit() {}
"#
}

/// Hostile: custom derive on a public type (opaque expansion).
pub fn hostile_custom_derive_on_public_type() -> &'static str {
    r#"
#[derive(Debug, CeremonyAuth)]
pub struct Carrier;
"#
}

/// Legal: standard library derives on a public type.
pub fn legal_std_derive_on_public_type() -> &'static str {
    r#"
#[derive(Clone, Copy, Debug, Eq, PartialEq, Hash)]
pub struct Token;
"#
}

/// Hostile: multi-hop use rename of AuthorityMarker used as a bound.
pub fn hostile_transitive_use_alias() -> &'static str {
    r#"
pub trait AuthorityMarker: 'static {}

use AuthorityMarker as AuthMark;
use AuthMark as AM;

pub fn admit<Auth: AM>(_authority: Auth) {}
"#
}

/// Hostile: private trait alias of AuthorityMarker used on a public ceremony.
pub fn hostile_private_trait_alias_bound() -> &'static str {
    r#"
pub trait AuthorityMarker: 'static {}

trait LocalAuth = AuthorityMarker;

pub fn admit<Auth: LocalAuth>(_authority: Auth) {}
"#
}

/// Hostile: parent-module alias re-bound in a child module on a public fn.
pub fn hostile_parent_alias_in_child_module() -> &'static str {
    r#"
pub trait AuthorityMarker: 'static {}

use AuthorityMarker as AuthMark;

mod inner {
    use super::AuthMark as AM;

    pub fn admit<Auth: AM>(_authority: Auth) {}
}

pub use inner::admit;
"#
}

/// Hostile: nested cfg_attr smuggles an opaque attribute onto a public item.
pub fn hostile_nested_cfg_attr_opaque() -> &'static str {
    r#"
#[cfg_attr(all(), cfg_attr(all(), ceremony_factory))]
pub fn admit() {}
"#
}

/// Hostile: nested cfg_attr smuggles a custom derive onto a public type.
pub fn hostile_nested_cfg_attr_custom_derive() -> &'static str {
    r#"
#[cfg_attr(any(), derive(CeremonyAuth))]
pub struct Carrier;
"#
}

/// Legal: cfg_attr carrying only known-safe nested attributes.
pub fn legal_cfg_attr_safe_nested() -> &'static str {
    r#"
#[cfg_attr(target_os = "windows", inline)]
#[cfg_attr(debug_assertions, allow(dead_code))]
pub fn admit() {}
"#
}

/// Hostile: public type alias exposes inherent methods on a private underlying type.
pub fn hostile_type_alias_owned_method() -> &'static str {
    r#"
pub trait AuthorityMarker: 'static {}

struct HiddenCeremony;

impl HiddenCeremony {
    pub fn admit(_authority: impl AuthorityMarker) {}
}

pub type PublicCeremony = HiddenCeremony;
"#
}

/// Hostile: nested type-alias chain still exposes underlying method ceremony.
pub fn hostile_type_alias_chain_owned_method() -> &'static str {
    r#"
pub trait AuthorityMarker: 'static {}

mod inner {
    use super::AuthorityMarker;

    struct Hidden;
    pub type Mid = Hidden;
    pub type Public = Mid;

    impl Hidden {
        pub fn admit(_authority: impl AuthorityMarker) {}
    }
}

pub use inner::Public;
"#
}

/// Hostile: named enum variant fields are ordinary public carriers.
pub fn hostile_enum_named_field_carrier() -> &'static str {
    r#"
pub trait AuthorityMarker: 'static {}

pub enum Gate {
    Open { authority: &'static dyn AuthorityMarker },
}
"#
}

/// Hostile: public extern crate re-exports a foreign root as ordinary public API.
pub fn hostile_pub_extern_crate() -> &'static str {
    r#"
pub extern crate core;

pub fn seed() {}
"#
}

/// Legal: private type alias + private underlying method stay ungoverned.
pub fn legal_private_type_alias_method() -> &'static str {
    r#"
pub trait AuthorityMarker: 'static {}

struct Hidden;

impl Hidden {
    pub fn admit(_authority: impl AuthorityMarker) {}
}

type Local = Hidden;

pub fn seed() {}
"#
}
