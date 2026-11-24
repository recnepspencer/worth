use contracts::schema::continuity::transition_lowering::lower_schema_transition;
use contracts::schema::continuity::transition_validation::validate_schema_transition;
use contracts::schema::data::continuity::descriptor_versions::DescriptorSemanticsVersion;
use contracts::schema::data::continuity::transition_plans::LoweredSchemaTransitionPlan;
use contracts::schema::data::continuity::transition_plans::SchemaContinuationDescriptor;
use contracts::schema::data::continuity::transition_plans::SchemaReconciliationDescriptor;
use contracts::schema::data::continuity::transition_plans::SchemaTransitionArtifact;
use contracts::transactions::data::outcomes::commit_conflict::CommitConflict;
use contracts::transactions::data::outcomes::commit_conflict::CommitConflictOwnerPort as _;
use contracts::transactions::data::outcomes::conflict_class::ConflictClass;
use contracts::transactions::data::outcomes::transaction_commit_error::TransactionCommitError;
use worth_relational_contracts::internal_port as contracts;
#[path = "schema_continuity_diagnostics/mod.rs"]
pub mod diagnostics;
pub(crate) use diagnostics::prepare_schema_continuity_diagnostics;
use diagnostics::{
    emit_schema_continuity_failure_diagnostic, schema_continuity_conflict_from_issue,
    FailureTransitionView,
};
pub mod inputs;
pub mod prior;
pub mod schema_transition_metrics;
use crate::publication::authority::diagnostic_emission::PreparationDiagnostics;
use prior::SchemaContinuityPrior;

#[derive(Debug, Clone)]
pub struct SchemaContinuityPlan {
    pub target_schema_registry:
        Option<worth_relational_branch::internal_port::branch::root::schema::compiled_retention::RetainedRelationalSchemaRegistry>,
    pub target_schema_version: contracts::schema::data::SchemaVersionId,
    pub target_schema_authority: contracts::schema::data::authority_snapshot::carried_projection::SchemaAuthoritySnapshot,
    pub descriptor_semantics_version: DescriptorSemanticsVersion,
    pub schema_transition: Option<SchemaTransitionArtifact>,
    pub schema_continuation_descriptor: Option<SchemaContinuationDescriptor>,
    pub schema_reconciliation_descriptor: Option<SchemaReconciliationDescriptor>,
}

impl SchemaContinuityPlan {
    pub(crate) fn current(
        input: &worth_relational_branch::internal_port::schema::continuity::authority_input::SchemaContinuityAuthorityInput,
    ) -> Self {
        Self {
            target_schema_registry: input.target_schema_registry().cloned(),
            target_schema_version: input.target_schema_version(),
            target_schema_authority: input.target_schema_authority().clone(),
            descriptor_semantics_version: input.descriptor_semantics_version(),
            schema_transition: None,
            schema_continuation_descriptor: None,
            schema_reconciliation_descriptor: None,
        }
    }

    pub fn target_schema_registry(&self) -> Option<&worth_relational_branch::internal_port::branch::root::schema::compiled_retention::RetainedRelationalSchemaRegistry>{
        self.target_schema_registry.as_ref()
    }

    pub(crate) const fn target_schema_version(&self) -> contracts::schema::data::SchemaVersionId {
        self.target_schema_version
    }

    pub(crate) fn target_schema_authority(
        &self,
    ) -> &contracts::schema::data::authority_snapshot::carried_projection::SchemaAuthoritySnapshot
    {
        &self.target_schema_authority
    }
}

#[path = "schema_continuity/resolution.rs"]
pub mod resolution;
pub use resolution::resolve_schema_continuity;

fn materialize_declared_transition(
    diagnostics: PreparationDiagnostics<'_>,
    proposed_transition: contracts::schema::data::continuity::transition_plans::ProposedSchemaTransition,
    policy: Option<
        contracts::schema::data::continuity::transition_classification::SchemaReconciliationPolicy,
    >,
    descriptor_semantics_version: DescriptorSemanticsVersion,
    descriptor_canonical_basis_version: contracts::schema::data::continuity::descriptor_versions::DescriptorCanonicalBasisVersion,
    branch_id: &contracts::history::data::coordinates::BranchId,
    previous_envelope: Option<&SchemaContinuityPrior<'_>>,
    current_schema_basis: Option<(
        contracts::schema::data::SchemaId,
        contracts::schema::data::SchemaVersionId,
    )>,
    current_schema_version: contracts::schema::data::SchemaVersionId,
    authority_input: &worth_relational_branch::internal_port::schema::continuity::authority_input::SchemaContinuityAuthorityInput,
) -> Result<SchemaContinuityPlan, TransactionCommitError> {
    if let Some(previous_envelope) = previous_envelope {
        let previous_schema_basis = previous_envelope
            .authority
            .registry()
            .authoritative_schema_basis()
            .ok()
            .flatten();
        let Some((previous_schema_id, previous_schema_version_id)) = previous_schema_basis else {
            return Err(schema_continuity_conflict(
                diagnostics,
                branch_id,
                Some(&proposed_transition),
                Some(previous_envelope),
                ConflictClass::MissingSchemaBasisForTransition {
                    role: "prior".to_string(),
                },
            ));
        };
        if proposed_transition.source_schema_id != previous_schema_id
            || proposed_transition.source_schema_version_id != previous_schema_version_id
        {
            return Err(schema_continuity_conflict(
                diagnostics,
                branch_id,
                Some(&proposed_transition),
                Some(previous_envelope),
                ConflictClass::InvalidSchemaTransitionSourceBasis {
                    declared_schema_id: proposed_transition.source_schema_id.clone(),
                    declared_schema_version: proposed_transition.source_schema_version_id,
                    expected_schema_id: previous_schema_id,
                    expected_schema_version: previous_schema_version_id,
                },
            ));
        }
    }

    let Some((current_schema_id, current_schema_version_id)) = current_schema_basis else {
        return Err(schema_continuity_conflict(
            diagnostics,
            branch_id,
            Some(&proposed_transition),
            previous_envelope,
            ConflictClass::MissingSchemaBasisForTransition {
                role: "runtime".to_string(),
            },
        ));
    };
    if proposed_transition.target_schema_id != current_schema_id
        || proposed_transition.target_schema_version_id != current_schema_version_id
        || proposed_transition.target_schema_version_id != current_schema_version
    {
        return Err(schema_continuity_conflict(
            diagnostics,
            branch_id,
            Some(&proposed_transition),
            previous_envelope,
            ConflictClass::InvalidSchemaTransitionTargetBasis {
                declared_schema_id: proposed_transition.target_schema_id.clone(),
                declared_schema_version: proposed_transition.target_schema_version_id,
                expected_schema_id: current_schema_id,
                expected_schema_version: current_schema_version_id,
            },
        ));
    }

    let validated =
        validate_schema_transition(proposed_transition.clone(), policy).map_err(|error| {
            schema_continuity_conflict(
                diagnostics,
                branch_id,
                Some(&proposed_transition),
                previous_envelope,
                ConflictClass::InvalidSchemaTransitionShape {
                    detail: error.detail(),
                },
            )
        })?;
    match validated.reconciliation {
        contracts::schema::data::continuity::transition_classification::SchemaReconciliationClassification::TypeContinuityDenied => {
            return Err(schema_continuity_conflict(
                diagnostics,
                branch_id,
                Some(&proposed_transition),
                previous_envelope,
                ConflictClass::TypeContinuityDeniedSchemaTransition {
                    detail: "declared schema transition contains a type-continuity-denied boundary that cannot continue honestly"
                        .to_string(),
                },
            ));
        }
        contracts::schema::data::continuity::transition_classification::SchemaReconciliationClassification::StructuralContinuityDenied => {
            return Err(schema_continuity_conflict(
                diagnostics,
                branch_id,
                Some(&proposed_transition),
                previous_envelope,
                ConflictClass::StructuralContinuityDeniedSchemaTransition {
                    detail: "declared schema transition contains a structural/semantic continuity denial that cannot continue honestly"
                        .to_string(),
                },
            ));
        }
        _ => {}
    }
    let lowered = lower_schema_transition(
        validated,
        policy,
        descriptor_semantics_version,
        descriptor_canonical_basis_version,
    );
    let atoms_inspected = proposed_transition.diff_atoms.len();
    // Milestone 5 does not yet reuse unchanged subtrees by fingerprint, so each
    // diff atom is currently both the inspected atom and the inspected change unit.
    let changed_subtrees_inspected = proposed_transition.diff_atoms.len();
    let unchanged_subtrees_reused_by_fingerprint = 0;
    schema_transition_metrics::count_schema_transition_classification(
        diagnostics.instrumentation,
        atoms_inspected,
        changed_subtrees_inspected,
        unchanged_subtrees_reused_by_fingerprint,
    );
    schema_transition_metrics::count_schema_bridge_descriptor(
        diagnostics.instrumentation,
        lowered.continuation_descriptor.bridge.continuation,
        lowered
            .continuation_descriptor
            .bridge
            .historical_interpretation,
        lowered.reconciliation_descriptor.policy,
    );
    Ok(schema_continuity_plan_from_lowered(
        proposed_transition,
        lowered,
        descriptor_semantics_version,
        branch_id,
        authority_input,
    ))
}

fn schema_continuity_plan_from_lowered(
    proposed_transition: contracts::schema::data::continuity::transition_plans::ProposedSchemaTransition,
    lowered: LoweredSchemaTransitionPlan,
    descriptor_semantics_version: DescriptorSemanticsVersion,
    branch_id: &contracts::history::data::coordinates::BranchId,
    authority_input: &worth_relational_branch::internal_port::schema::continuity::authority_input::SchemaContinuityAuthorityInput,
) -> SchemaContinuityPlan {
    let continuation_descriptor = lowered.continuation_descriptor.clone();
    let mut reconciliation_descriptor = lowered.reconciliation_descriptor.clone();
    reconciliation_descriptor.resulting_lineage.branch_context = Some(branch_id.clone());
    let schema_transition = SchemaTransitionArtifact::new(
        proposed_transition.source_schema_id,
        proposed_transition.source_schema_version_id,
        proposed_transition.target_schema_id,
        proposed_transition.target_schema_version_id,
        proposed_transition.diff_atoms,
        continuation_descriptor.clone(),
        reconciliation_descriptor.clone(),
    );

    SchemaContinuityPlan {
        target_schema_registry: authority_input.target_schema_registry().cloned(),
        target_schema_version: authority_input.target_schema_version(),
        target_schema_authority: authority_input.target_schema_authority().clone(),
        descriptor_semantics_version,
        schema_transition: Some(schema_transition),
        schema_continuation_descriptor: Some(continuation_descriptor),
        schema_reconciliation_descriptor: Some(reconciliation_descriptor),
    }
}

fn schema_continuity_conflict(
    diagnostics: PreparationDiagnostics<'_>,
    branch_id: &contracts::history::data::coordinates::BranchId,
    proposed_transition: Option<
        &contracts::schema::data::continuity::transition_plans::ProposedSchemaTransition,
    >,
    previous_envelope: Option<&SchemaContinuityPrior<'_>>,
    class: ConflictClass,
) -> TransactionCommitError {
    let conflict = CommitConflict::new(class);
    emit_schema_continuity_failure_diagnostic(
        diagnostics,
        branch_id,
        proposed_transition.map(FailureTransitionView::Proposed),
        previous_envelope,
        &conflict,
    );
    TransactionCommitError::conflict(conflict)
}

pub fn validate_schema_continuity_publication(
    diagnostics: PreparationDiagnostics<'_>,
    branch_id: &contracts::history::data::coordinates::BranchId,
    _plan: &SchemaContinuityPlan,
    envelope: &worth_relational_branch::internal_port::history::data::canonical_commit_envelope::CanonicalCommitEnvelopeBody,
) -> Result<(), TransactionCommitError> {
    worth_relational_branch::internal_port::schema::continuity::bundle_admission::validate_schema_continuity_body(envelope).map_err(|issue| {
        schema_continuity_conflict_from_issue(
            diagnostics,
            branch_id,
            envelope.schema_transition.as_ref(),
            issue,
            envelope,
        )
    })?;

    Ok(())
}
