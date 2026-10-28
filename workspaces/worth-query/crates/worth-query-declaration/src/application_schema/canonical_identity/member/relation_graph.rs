//! Canonical graph-law emission shared without stored deletion semantics.
use crate::application_schema::canonical_basis::ApplicationSchemaCanonicalBasis;
use crate::application_schema::relation_integrity::{
    ApplicationRelationGraphLawRef, ApplicationRelationUniquenessScope,
};
use crate::application_schema::ApplicationRelationCrossContextPolicy as CrossContext;

pub(super) fn append(
    basis: &mut ApplicationSchemaCanonicalBasis,
    prefix: &str,
    laws: ApplicationRelationGraphLawRef<'_>,
    unique: Option<ApplicationRelationUniquenessScope>,
) {
    // No enabled law preserves the pre-existing relation meaning.
    for (law, value) in laws.enabled_laws(unique) {
        use crate::application_schema::relation_integrity::ApplicationRelationLawValue;
        match value {
            ApplicationRelationLawValue::Number(value) => {
                basis.u64(format!("{prefix}.laws.{law}"), value)
            }
            value @ ApplicationRelationLawValue::KindSet(_) => {
                basis.text(format!("{prefix}.laws.{law}"), &value.canonical_text())
            }
        }
    }
    basis.text(
        format!("{prefix}.endpoints.cross_context"),
        match laws.endpoints.cross_context_policy {
            CrossContext::AllowExplicit => "allow-explicit",
            CrossContext::SchemaControlled => "schema-controlled",
            CrossContext::Forbid => "forbid",
        },
    );
    basis.u64(
        format!("{prefix}.endpoints.self_edges_allowed"),
        u64::from(laws.endpoints.self_edges_allowed),
    );
    for (name, value) in [
        ("source_min", laws.cardinality.source_min),
        ("source_max", laws.cardinality.source_max),
        ("target_min", laws.cardinality.target_min),
        ("target_max", laws.cardinality.target_max),
        ("pair_min", laws.cardinality.pair_min),
        ("pair_max", laws.cardinality.pair_max),
    ] {
        basis.text(
            format!("{prefix}.cardinality.{name}.presence"),
            if value.is_some() { "some" } else { "none" },
        );
        if let Some(value) = value {
            basis.u64(format!("{prefix}.cardinality.{name}.value"), value);
        }
    }
}
