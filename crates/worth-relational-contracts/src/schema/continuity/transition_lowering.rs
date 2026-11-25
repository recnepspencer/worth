use crate::schema::data::{
    DescriptorCanonicalBasisVersion, DescriptorSemanticsVersion,
    HistoricalInterpretationSensitivity, LoweredSchemaTransitionPlan, SchemaBridgeDescriptor,
    SchemaContinuationClassification, SchemaContinuationDescriptor, SchemaLineageArtifact,
    SchemaLineageOrderingSemantics, SchemaReconciliationDescriptor,
    SchemaReconciliationOrderingMode, SchemaReconciliationPolicy, ValidatedSchemaTransition,
};

use crate::schema::continuity::transition_canonical_fingerprint::fingerprint_writer::fingerprint_transition;
use crate::schema::continuity::transition_canonical_fingerprint::transition_strength::{
    strongest_boundary_visibility, strongest_historical_interpretation,
};
use crate::schema::continuity::transition_classification::is_contract_upgrade_policy;

pub fn lower_schema_transition(
    validated: ValidatedSchemaTransition,
    policy: Option<SchemaReconciliationPolicy>,
    semantics_version: DescriptorSemanticsVersion,
    canonical_basis_version: DescriptorCanonicalBasisVersion,
) -> LoweredSchemaTransitionPlan {
    let fingerprint = fingerprint_transition(&validated.proposed.diff_atoms);
    let bridge = SchemaBridgeDescriptor::new_with_visibility(
        fingerprint,
        semantics_version,
        canonical_basis_version,
        validated.continuation,
        bridgeability_after_policy(&validated, policy),
        strongest_boundary_visibility(&validated.proposed.diff_atoms),
        strongest_transition_historical_interpretation(&validated),
        changed_transition_strata(&validated),
    );
    let continuation_descriptor =
        SchemaContinuationDescriptor::new(fingerprint, bridge, validated.proposed.diff_atoms.len());
    let reconciliation_descriptor = SchemaReconciliationDescriptor::new(
        semantics_version,
        canonical_basis_version,
        validated.reconciliation,
        policy.unwrap_or(SchemaReconciliationPolicy::RejectLossyNarrowing),
        SchemaLineageArtifact::new(
            validated.proposed.target_schema_id.clone(),
            validated.proposed.target_schema_version_id,
            vec![validated.proposed.source_schema_id.clone()],
            vec![validated.proposed.source_schema_version_id],
            None,
            SchemaReconciliationOrderingMode::CanonicalizedPair,
            SchemaLineageOrderingSemantics::SymmetricResult,
        ),
    );

    LoweredSchemaTransitionPlan::new(
        validated,
        continuation_descriptor,
        reconciliation_descriptor,
    )
}

fn bridgeability_after_policy(
    validated: &ValidatedSchemaTransition,
    policy: Option<SchemaReconciliationPolicy>,
) -> crate::schema::data::SchemaBridgeabilityClassification {
    if is_contract_upgrade_policy(policy)
        && validated.continuation == SchemaContinuationClassification::ContinueWithContractUpgrade
    {
        crate::schema::data::SchemaBridgeabilityClassification::ContractUpgradeOnly
    } else {
        validated.bridgeability
    }
}

fn changed_transition_strata(
    validated: &ValidatedSchemaTransition,
) -> Vec<crate::schema::data::SchemaStratum> {
    let mut changed_strata = std::collections::BTreeSet::new();
    for atom in &validated.proposed.diff_atoms {
        for stratum in &atom.strata {
            changed_strata.insert(*stratum);
        }
    }
    changed_strata.into_iter().collect()
}

fn strongest_transition_historical_interpretation(
    validated: &ValidatedSchemaTransition,
) -> HistoricalInterpretationSensitivity {
    let mut historical_interpretation = HistoricalInterpretationSensitivity::NotSensitive;
    for atom in &validated.proposed.diff_atoms {
        historical_interpretation = strongest_historical_interpretation(
            historical_interpretation,
            atom.historical_interpretation,
        );
    }
    historical_interpretation
}
