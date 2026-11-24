use contracts::diagnostics::data::artifacts::DiagnosticCode;
use contracts::diagnostics::data::artifacts::RelationalDiagnosticsEntry;
use contracts::history::data::coordinates::BranchId;
use contracts::schema::data::continuity::transition_plans::SchemaTransitionArtifact;
use worth_relational_contracts::internal_port as contracts;

use super::diagnostic_fields::diagnostics_fields;
use super::field_shapes::{
    schema_diff_atom_trace_fields, SchemaBridgeDescriptorFields, SchemaDescriptorVersionFields,
    SchemaInterpretationFields, SchemaLineageFields, SchemaReconciliationFields,
    SchemaTransitionClassificationFields,
};

pub(super) fn schema_transition_trace_entries(
    branch_id: &BranchId,
    transition: &SchemaTransitionArtifact,
) -> Vec<RelationalDiagnosticsEntry> {
    let mut entries = core_schema_transition_trace_entries(branch_id, transition);

    entries.extend(
        transition
            .diff_atoms
            .iter()
            .enumerate()
            .map(|(index, atom)| {
                RelationalDiagnosticsEntry::new(
                    DiagnosticCode::SchemaTransitionClassified,
                    format!("schema diff atom {index} classified for continuity"),
                    diagnostics_fields(schema_diff_atom_trace_fields(index, atom)),
                )
            }),
    );

    entries
}

fn core_schema_transition_trace_entries(
    branch_id: &BranchId,
    transition: &SchemaTransitionArtifact,
) -> Vec<RelationalDiagnosticsEntry> {
    vec![
        bridge_descriptor_entry(transition),
        interpretation_sensitivity_entry(transition),
        reconciliation_entry(transition),
        descriptor_version_entry(transition),
        transition_classification_entry(branch_id, transition),
        lineage_entry(transition),
    ]
}

fn bridge_descriptor_entry(transition: &SchemaTransitionArtifact) -> RelationalDiagnosticsEntry {
    RelationalDiagnosticsEntry::new(
        DiagnosticCode::SchemaBridgeDescriptorConstructed,
        "schema bridge descriptor constructed for continuity boundary",
        diagnostics_fields(SchemaBridgeDescriptorFields {
            boundary_fingerprint: transition.continuation_descriptor.boundary_fingerprint,
            continuation: format!(
                "{:?}",
                transition.continuation_descriptor.bridge.continuation
            ),
            bridgeability: format!(
                "{:?}",
                transition.continuation_descriptor.bridge.bridgeability
            ),
            normalized_boundary_count: transition.continuation_descriptor.normalized_boundary_count,
            descriptor_canonical_basis_version: transition
                .continuation_descriptor
                .bridge
                .canonical_basis_version,
        }),
    )
}

fn interpretation_sensitivity_entry(
    transition: &SchemaTransitionArtifact,
) -> RelationalDiagnosticsEntry {
    RelationalDiagnosticsEntry::new(
        DiagnosticCode::SchemaInterpretationSensitivityClassified,
        "schema historical interpretation sensitivity classified",
        diagnostics_fields(SchemaInterpretationFields {
            boundary_fingerprint: transition.continuation_descriptor.boundary_fingerprint,
            historical_interpretation: format!(
                "{:?}",
                transition
                    .continuation_descriptor
                    .bridge
                    .historical_interpretation
            ),
            changed_strata: transition
                .continuation_descriptor
                .bridge
                .changed_strata
                .clone(),
        }),
    )
}

fn reconciliation_entry(transition: &SchemaTransitionArtifact) -> RelationalDiagnosticsEntry {
    RelationalDiagnosticsEntry::new(
        DiagnosticCode::SchemaReconciliationResolved,
        "schema reconciliation result resolved for continuity boundary",
        diagnostics_fields(SchemaReconciliationFields {
            classification: format!("{:?}", transition.reconciliation_descriptor.classification),
            policy: format!("{:?}", transition.reconciliation_descriptor.policy),
            resulting_schema_id: transition
                .reconciliation_descriptor
                .resulting_lineage
                .resulting_schema_id
                .clone(),
            resulting_schema_version_id: transition
                .reconciliation_descriptor
                .resulting_lineage
                .resulting_schema_version_id,
        }),
    )
}

fn descriptor_version_entry(transition: &SchemaTransitionArtifact) -> RelationalDiagnosticsEntry {
    RelationalDiagnosticsEntry::new(
        DiagnosticCode::SchemaDescriptorVersionSelected,
        "schema descriptor semantics version selected for continuity boundary",
        diagnostics_fields(SchemaDescriptorVersionFields {
            descriptor_semantics_version: transition
                .continuation_descriptor
                .bridge
                .semantics_version,
            continuation_canonical_basis_version: transition
                .continuation_descriptor
                .bridge
                .canonical_basis_version,
            reconciliation_canonical_basis_version: transition
                .reconciliation_descriptor
                .canonical_basis_version,
        }),
    )
}

fn transition_classification_entry(
    branch_id: &BranchId,
    transition: &SchemaTransitionArtifact,
) -> RelationalDiagnosticsEntry {
    RelationalDiagnosticsEntry::new(
        DiagnosticCode::SchemaTransitionClassified,
        "schema transition classified into continuation and reconciliation outcomes",
        diagnostics_fields(SchemaTransitionClassificationFields {
            branch_id: branch_id.clone(),
            boundary_fingerprint: transition.continuation_descriptor.boundary_fingerprint,
            continuation: format!(
                "{:?}",
                transition.continuation_descriptor.bridge.continuation
            ),
            bridgeability: format!(
                "{:?}",
                transition.continuation_descriptor.bridge.bridgeability
            ),
            historical_interpretation: format!(
                "{:?}",
                transition
                    .continuation_descriptor
                    .bridge
                    .historical_interpretation
            ),
            changed_strata: transition
                .continuation_descriptor
                .bridge
                .changed_strata
                .clone(),
            reconciliation: format!("{:?}", transition.reconciliation_descriptor.classification),
            policy: format!("{:?}", transition.reconciliation_descriptor.policy),
        }),
    )
}

fn lineage_entry(transition: &SchemaTransitionArtifact) -> RelationalDiagnosticsEntry {
    RelationalDiagnosticsEntry::new(
        DiagnosticCode::SchemaLineageTraced,
        "schema reconciliation lineage recorded for continuity boundary",
        diagnostics_fields(SchemaLineageFields {
            resulting_schema_id: transition
                .reconciliation_descriptor
                .resulting_lineage
                .resulting_schema_id
                .clone(),
            resulting_schema_version_id: transition
                .reconciliation_descriptor
                .resulting_lineage
                .resulting_schema_version_id,
            parent_schema_ids: transition
                .reconciliation_descriptor
                .resulting_lineage
                .parent_schema_ids
                .clone(),
            parent_schema_version_ids: transition
                .reconciliation_descriptor
                .resulting_lineage
                .parent_schema_version_ids
                .clone(),
            ordering_mode: format!(
                "{:?}",
                transition
                    .reconciliation_descriptor
                    .resulting_lineage
                    .ordering_mode
            ),
            ordering_semantics: format!(
                "{:?}",
                transition
                    .reconciliation_descriptor
                    .resulting_lineage
                    .ordering_semantics
            ),
            branch_context: transition
                .reconciliation_descriptor
                .resulting_lineage
                .branch_context
                .clone(),
        }),
    )
}
