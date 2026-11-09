//! Pure index preimages; these comparisons grant no schema or Source admission.
use super::*;
use crate::facade::{
    application_query::*, application_schema::*,
    portable_identity::WorthQueryPortableTypeIdentity as Identity,
};

#[derive(worth_foundational::facade::WorthValue)]
struct Probe {
    prefix: String,
}
crate::worth_query_value_binding!(ProbeBinding for Probe {
    identity: "worth.tests.canonical-index-probe.v1", scalar: String,
    canonical: worth_foundational::facade::ValueDecodeBounds::new(1024,512,4,4),
});

fn parts(version: u16) -> WorthQueryPortableManagedIndexParts {
    WorthQueryPortableManagedIndexParts {
        semantic_version: version,
        name: "Prefix".into(),
        provider: "prefix-v1".into(),
        provider_version: 1,
        structure: if version == 1 {
            ApplicationManagedIndexSourceStructure::Opaque
        } else {
            ApplicationManagedIndexSourceStructure::BinarySourceTree
        },
        membership: if version < 3 {
            ApplicationManagedIndexMembership::Candidates
        } else {
            ApplicationManagedIndexMembership::CheckedSummarySourceTree
        },
        source: ApplicationManagedIndexSourceMeaning::same_scope_without_parameters(
            ApplicationQueryBindingReference::from_untrusted_parts(
                ApplicationQueryBindingReferenceParts {
                    binding: Identity::declared("worth.tests.source-binding.v1"),
                    input: Identity::declared("worth.tests.source-input.v1"),
                    query_name: "Source".into(),
                    query: Identity::declared("worth.tests.source-query.v1"),
                    parameters: Identity::declared("worth.rust.unit"),
                    result: Identity::declared("worth.tests.row.v1"),
                    scope: Identity::declared("worth.tests.scope.v1"),
                    scope_entity: "Directory".into(),
                    scope_aspect: "Identity".into(),
                    scope_field: "Id".into(),
                    scope_value: Identity::declared("worth.rust.u64"),
                    scope_resolution: ApplicationQueryScopeResolutionMode::PrincipalIdentity,
                    principal_binding: "Principal".into(),
                },
            ),
        ),
        entity: "Item".into(),
        probe: ApplicationManagedIndexProbeMeaning::of::<ProbeBinding>(),
    }
}
fn identity(
    definition: ErasedApplicationManagedIndexDefinition,
) -> super::super::ApplicationSchemaIdentity {
    definition.validate().unwrap();
    canonical_identity(
        header(),
        &[ApplicationSchemaMember::ManagedIndex { definition }],
        &[],
    )
}

#[test]
fn explicit_maintenance_cold_and_optional_refinement_have_distinct_canonical_preimages() {
    use ApplicationManagedIndexColdSource as Cold;
    use ApplicationManagedIndexMaintenance as Maintenance;
    let refinement = ApplicationManagedIndexRefinement::row_local("domain.exact.v1", 1).meaning();
    let identities = [
        (Maintenance::WhenRequired, None),
        (Maintenance::Native { cold: Cold::Refuse }, None),
        (
            Maintenance::Native {
                cold: Cold::AdmitRebuild,
            },
            None,
        ),
        (
            Maintenance::Native {
                cold: Cold::AdmitRebuild,
            },
            Some(refinement),
        ),
    ]
    .map(|(maintenance, refinement)| {
        identity(
            ErasedApplicationManagedIndexDefinition::from_maintenance_parts(
                parts(5),
                maintenance,
                refinement,
            ),
        )
    });
    for (i, left) in identities.iter().enumerate() {
        for right in &identities[i + 1..] {
            assert_ne!(left, right);
        }
    }
    for version in 1..=4 {
        let definition = if version == 4 {
            ErasedApplicationManagedIndexDefinition::from_refinement_parts(
                parts(version),
                ApplicationManagedIndexRefinement::row_local("domain.exact.v1", 1).meaning(),
            )
        } else {
            ErasedApplicationManagedIndexDefinition::from_untrusted_parts(parts(version))
        };
        assert_eq!(definition.maintenance(), Maintenance::WhenRequired);
        let identity = identity(definition);
        assert_eq!(
            identity.canonical_basis().payload().version().as_str(),
            "worth-query-application-schema-v18"
        );
        assert!(!identity.canonical_basis().payload().entries().iter().any(|entry|
            matches!(entry.locus(),worth_foundational::facade::CanonicalBasisLocus::Named(worth_foundational::facade::InternedString::Raw(name)) if name.contains("public-maintenance") || name.ends_with("refinement.present"))));
    }
}
