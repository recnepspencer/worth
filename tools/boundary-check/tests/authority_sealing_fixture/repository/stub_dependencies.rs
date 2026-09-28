//! Stub packages required by authority sealing fixture repositories.

use super::AuthoritySealingTestRepository;

/// A documented engine whose items the audience leaf facades re-export, so
/// the facade documentation ratchet resolves every leaf name.
const STUB_ENGINE: &str = r#"//! Stub Query engine.

/// Stub engine facade.
pub mod facade {
    /// Stub foundation vocabulary.
    pub mod foundation {
        /// Stub canonical artifact.
        pub struct CanonicalQueryArtifact;
        /// Stub replay basis.
        pub struct ScopedReplayBasis;
    }
    /// Stub domain namespace.
    pub mod domain {}
    /// Stub runtime namespace.
    pub mod runtime {}
}
"#;

impl AuthoritySealingTestRepository {
    pub(super) fn write_worth_proof_stub(&self) {
        self.write_file(
            "vendor/worth-proof/Cargo.toml",
            r#"[package]
name = "worth-proof"
version = "0.1.0"
edition = "2021"
[workspace]
"#,
        );
        self.write_file(
            "vendor/worth-proof/src/lib.rs",
            r#"pub struct AuthorityWitness<A>(core::marker::PhantomData<A>);
pub struct CapabilityWitness<C>(core::marker::PhantomData<C>);
pub struct Proof<P, A>(core::marker::PhantomData<(P, A)>);
"#,
        );
    }

    pub(super) fn write_query_audience_leaf_facades(&self) {
        self.write_file(
            "crates/worth-query/Cargo.toml",
            r#"[package]
name = "worth-query"
version = "0.1.0"
edition = "2021"
[workspace]
"#,
        );
        self.write_file("crates/worth-query/src/lib.rs", STUB_ENGINE);
        for (package, export) in [
            (
                "worth-query-decl",
                "pub use worth_query::facade::foundation::CanonicalQueryArtifact;\n",
            ),
            (
                "worth-query-host",
                "pub use worth_query::facade::domain;\npub use worth_query::facade::runtime;\n",
            ),
            (
                "worth-query-replay",
                "pub use worth_query::facade::foundation::ScopedReplayBasis;\n",
            ),
        ] {
            self.write_file(
                &format!("crates/{package}/Cargo.toml"),
                &format!(
                    r#"[package]
name = "{package}"
version = "0.1.0"
edition = "2021"

[dependencies]
worth-query = {{ path = "../worth-query" }}

[workspace]
"#
                ),
            );
            self.write_file(&format!("crates/{package}/src/lib.rs"), "pub mod facade;\n");
            self.write_file(&format!("crates/{package}/src/facade.rs"), export);
        }
    }
}
