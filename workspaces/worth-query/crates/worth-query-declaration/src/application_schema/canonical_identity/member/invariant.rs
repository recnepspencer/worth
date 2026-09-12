use crate::application_schema::canonical_basis::ApplicationSchemaCanonicalBasis;
use crate::application_schema::{
    ApplicationInvariantCostPosture, ApplicationInvariantEnforcement,
    ApplicationInvariantExecutionPoint, ApplicationInvariantGroup, ApplicationInvariantScopeTarget,
    ApplicationSchemaMember,
};

pub(super) fn append(
    basis: &mut ApplicationSchemaCanonicalBasis,
    prefix: &str,
    member: &ApplicationSchemaMember,
) {
    let ApplicationSchemaMember::ApplicationInvariant {
        invariant,
        major,
        minor,
        execution_point,
        maximum_work_units,
        enforcement,
        required_groups,
        read_closure,
        applicability,
        provider,
        cost_posture,
    } = member
    else {
        unreachable!("invariant canonicalizer receives invariant member")
    };
    basis.text(format!("{prefix}.kind"), "application-invariant");
    basis.text(format!("{prefix}.identity"), invariant);
    basis.u32(format!("{prefix}.major"), u32::from(*major));
    basis.u32(format!("{prefix}.minor"), u32::from(*minor));
    basis.text(
        format!("{prefix}.execution-point"),
        match execution_point {
            ApplicationInvariantExecutionPoint::CommitBoundary => "commit-boundary",
            ApplicationInvariantExecutionPoint::MutationSensitive => "mutation-sensitive",
            ApplicationInvariantExecutionPoint::SnapshotPublication => "snapshot-publication",
        },
    );
    basis.u64(
        format!("{prefix}.maximum-work-units"),
        maximum_work_units.get(),
    );
    basis.text(
        format!("{prefix}.enforcement"),
        enforcement_name(*enforcement),
    );
    basis.text(format!("{prefix}.provider"), provider);
    basis.text(format!("{prefix}.cost-posture"), cost_name(*cost_posture));
    basis.usize(
        format!("{prefix}.required-group-count"),
        required_groups.len(),
    );
    for (index, group) in required_groups.iter().enumerate() {
        basis.text(
            format!("{prefix}.required-group[{index}]"),
            group_name(*group),
        );
    }
    basis.usize(format!("{prefix}.read-closure-count"), read_closure.len());
    for (index, read) in read_closure.iter().enumerate() {
        basis.text(
            format!("{prefix}.read-closure[{index}]"),
            &target_name(read),
        );
    }
    basis.usize(format!("{prefix}.applicability-count"), applicability.len());
    for (index, scope) in applicability.iter().enumerate() {
        basis.text(
            format!("{prefix}.applicability[{index}]"),
            &target_name(scope),
        );
    }
}

fn enforcement_name(value: ApplicationInvariantEnforcement) -> &'static str {
    match value {
        ApplicationInvariantEnforcement::BlockCommit => "block-commit",
        ApplicationInvariantEnforcement::BlockPublication => "block-publication",
    }
}
fn cost_name(value: ApplicationInvariantCostPosture) -> &'static str {
    match value {
        ApplicationInvariantCostPosture::Touched => "touched",
        ApplicationInvariantCostPosture::Partition => "partition",
        ApplicationInvariantCostPosture::Global => "global",
    }
}
fn group_name(value: ApplicationInvariantGroup) -> &'static str {
    match value {
        ApplicationInvariantGroup::StorageCoherence => "storage-coherence",
        ApplicationInvariantGroup::VersionVisibility => "version-visibility",
        ApplicationInvariantGroup::AdjacencyIntegrity => "adjacency-integrity",
        ApplicationInvariantGroup::IdentityCoherence => "identity-coherence",
        ApplicationInvariantGroup::SchemaCompliance => "schema-compliance",
        ApplicationInvariantGroup::LineageIntegrity => "lineage-integrity",
        ApplicationInvariantGroup::PublicationCoherence => "publication-coherence",
        ApplicationInvariantGroup::RelationIntegrity => "relation-integrity",
    }
}
fn target_name(value: &ApplicationInvariantScopeTarget) -> String {
    match value {
        ApplicationInvariantScopeTarget::Entity(name) => format!("entity:{name}"),
        ApplicationInvariantScopeTarget::Relation(name) => format!("relation:{name}"),
    }
}
