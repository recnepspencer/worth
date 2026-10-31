use crate::schema::data::{
    DescriptorSemanticsVersion, LoweredSchemaTransitionPlan, SchemaContinuationDescriptor,
    SchemaReconciliationDescriptor, SchemaTransitionArtifact,
};
use crate::schema::{lower_schema_transition, validate_schema_transition};
use crate::transactions::data::{CommitConflict, ConflictClass, TransactionCommitError};
#[path = "schema_continuity_diagnostics/mod.rs"]
mod diagnostics;
pub(crate) use diagnostics::prepare_schema_continuity_diagnostics;
use diagnostics::{
    emit_schema_continuity_failure_diagnostic, schema_continuity_conflict_from_issue,
    FailureTransitionView,
};

#[derive(Debug, Clone)]
pub(crate) struct SchemaContinuityPlan {
    pub(crate) target_schema_registry:
        Option<std::sync::Arc<crate::schema::data::RelationalSchemaRegistry>>,
    pub(crate) target_schema_version: crate::schema::data::SchemaVersionId,
    pub(crate) target_schema_authority: crate::schema::data::SchemaAuthoritySnapshot,
    pub(crate) descriptor_semantics_version: DescriptorSemanticsVersion,
    pub(crate) schema_transition: Option<SchemaTransitionArtifact>,
    pub(crate) schema_continuation_descriptor: Option<SchemaContinuationDescriptor>,
    pub(crate) schema_reconciliation_descriptor: Option<SchemaReconciliationDescriptor>,
}

impl SchemaContinuityPlan {
    pub(crate) fn current(input: &crate::schema::SchemaContinuityAuthorityInput) -> Self {
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

    pub(crate) const fn target_schema_version(&self) -> crate::schema::data::SchemaVersionId {
        self.target_schema_version
    }

    pub(crate) fn target_schema_authority(&self) -> &crate::schema::data::SchemaAuthoritySnapshot {
        &self.target_schema_authority
    }
}

pub(crate) fn resolve_schema_continuity(
    runtime: &crate::runtime::RelationalPreparationRuntime,
    branch_id: &crate::history::data::BranchId,
    options: &crate::mvcc::RelationalTransactionValidationInput,
) -> Result<SchemaContinuityPlan, TransactionCommitError> {
    let descriptor_policy = runtime.config.schema.descriptor_semantics_policy.clone();
    let authority_input = match (
        options.schema_authority_input(),
        options.proposed_schema_transition(),
    ) {
        (Some(input), _) => input.clone(),
        (None, Some(_)) => crate::schema::SchemaContinuityAuthorityInput::from_shared_registry(
            std::sync::Arc::new(runtime.config.schema.registry.clone()),
            runtime
                .config
                .schema
                .descriptor_semantics_policy
                .current_write_version(),
            runtime
                .config
                .schema
                .descriptor_canonical_basis_policy
                .current_write_version(),
        ),
        (None, None) => crate::schema::SchemaContinuityAuthorityInput::from_shared_registry(
            options.schema_authority().registry_arc(),
            options.schema_authority().descriptor_semantics_version(),
            runtime
                .config
                .schema
                .descriptor_canonical_basis_policy
                .current_write_version(),
        ),
    };
    let current_descriptor_semantics_version = authority_input.descriptor_semantics_version();
    let current_descriptor_canonical_basis_version =
        authority_input.descriptor_canonical_basis_version();
    let current_schema_version = authority_input.target_schema_version();
    let current_schema_authority = authority_input.target_schema_authority().clone();
    let current_schema_basis = authority_input.target_schema_basis();
    let previous_envelope = runtime
        .history
        .branch_cell(branch_id)
        .and_then(|cell| cell.root())
        .and_then(|root| root.canonical_envelope().cloned());
    let Some(previous_envelope) = previous_envelope.as_ref() else {
        if let Some(proposed_transition) = options.proposed_schema_transition() {
            return materialize_declared_transition(
                runtime,
                proposed_transition.clone(),
                options.schema_reconciliation_policy().cloned(),
                current_descriptor_semantics_version,
                current_descriptor_canonical_basis_version,
                branch_id,
                None,
                current_schema_basis,
                current_schema_version,
                &authority_input,
            );
        }
        return Ok(SchemaContinuityPlan::current(&authority_input));
    };
    if !descriptor_policy.supports(previous_envelope.descriptor_semantics_version) {
        runtime
            .performance_access()
            .count_descriptor_version_mismatch();
        return Err(schema_continuity_conflict(
            runtime,
            branch_id,
            options.proposed_schema_transition(),
            Some(previous_envelope),
            ConflictClass::DescriptorSemanticsVersionUnsupported {
                previous_descriptor_semantics_version: previous_envelope
                    .descriptor_semantics_version,
                current_descriptor_semantics_version,
            },
        ));
    }

    let drift_detected = previous_envelope.schema_version != current_schema_version
        || previous_envelope.schema_authority != current_schema_authority;
    match options.proposed_schema_transition() {
        Some(proposed_transition) => materialize_declared_transition(
            runtime,
            proposed_transition.clone(),
            options.schema_reconciliation_policy().cloned(),
            current_descriptor_semantics_version,
            current_descriptor_canonical_basis_version,
            branch_id,
            Some(previous_envelope),
            current_schema_basis,
            current_schema_version,
            &authority_input,
        ),
        None if drift_detected => Err(schema_continuity_conflict(
            runtime,
            branch_id,
            None,
            Some(previous_envelope),
            ConflictClass::UndeclaredSchemaTransition {
                previous_schema_version: previous_envelope.schema_version,
                current_schema_version,
                previous_descriptor_semantics_version: previous_envelope
                    .descriptor_semantics_version,
                current_descriptor_semantics_version,
            },
        )),
        None => Ok(SchemaContinuityPlan::current(&authority_input)),
    }
}

fn materialize_declared_transition(
    runtime: &crate::runtime::RelationalPreparationRuntime,
    proposed_transition: crate::schema::data::ProposedSchemaTransition,
    policy: Option<crate::schema::data::SchemaReconciliationPolicy>,
    descriptor_semantics_version: DescriptorSemanticsVersion,
    descriptor_canonical_basis_version: crate::schema::data::DescriptorCanonicalBasisVersion,
    branch_id: &crate::history::data::BranchId,
    previous_envelope: Option<&crate::history::data::CanonicalCommitEnvelope>,
    current_schema_basis: Option<(
        crate::schema::data::SchemaId,
        crate::schema::data::SchemaVersionId,
    )>,
    current_schema_version: crate::schema::data::SchemaVersionId,
    authority_input: &crate::schema::SchemaContinuityAuthorityInput,
) -> Result<SchemaContinuityPlan, TransactionCommitError> {
    if let Some(previous_envelope) = previous_envelope {
        let previous_schema_basis = previous_envelope
            .schema_authority
            .primary_schema_id
            .clone()
            .zip(previous_envelope.schema_authority.primary_schema_version_id);
        let Some((previous_schema_id, previous_schema_version_id)) = previous_schema_basis else {
            return Err(schema_continuity_conflict(
                runtime,
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
                runtime,
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
            runtime,
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
            runtime,
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
                runtime,
                branch_id,
                Some(&proposed_transition),
                previous_envelope,
                ConflictClass::InvalidSchemaTransitionShape {
                    detail: error.detail(),
                },
            )
        })?;
    match validated.reconciliation {
        crate::schema::data::SchemaReconciliationClassification::TypeContinuityDenied => {
            return Err(schema_continuity_conflict(
                runtime,
                branch_id,
                Some(&proposed_transition),
                previous_envelope,
                ConflictClass::TypeContinuityDeniedSchemaTransition {
                    detail: "declared schema transition contains a type-continuity-denied boundary that cannot continue honestly"
                        .to_string(),
                },
            ));
        }
        crate::schema::data::SchemaReconciliationClassification::StructuralContinuityDenied => {
            return Err(schema_continuity_conflict(
                runtime,
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
    runtime
        .performance_access()
        .count_schema_transition_classification(
            atoms_inspected,
            changed_subtrees_inspected,
            unchanged_subtrees_reused_by_fingerprint,
        );
    runtime.performance_access().count_schema_bridge_descriptor(
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
    proposed_transition: crate::schema::data::ProposedSchemaTransition,
    lowered: LoweredSchemaTransitionPlan,
    descriptor_semantics_version: DescriptorSemanticsVersion,
    branch_id: &crate::history::data::BranchId,
    authority_input: &crate::schema::SchemaContinuityAuthorityInput,
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
    runtime: &crate::runtime::RelationalPreparationRuntime,
    branch_id: &crate::history::data::BranchId,
    proposed_transition: Option<&crate::schema::data::ProposedSchemaTransition>,
    previous_envelope: Option<&crate::history::data::CanonicalCommitEnvelope>,
    class: ConflictClass,
) -> TransactionCommitError {
    let conflict = CommitConflict::new(class);
    emit_schema_continuity_failure_diagnostic(
        runtime,
        branch_id,
        proposed_transition.map(FailureTransitionView::Proposed),
        previous_envelope,
        &conflict,
    );
    TransactionCommitError::conflict(conflict)
}

pub(crate) fn validate_schema_continuity_publication(
    runtime: &crate::runtime::RelationalPreparationRuntime,
    branch_id: &crate::history::data::BranchId,
    _plan: &SchemaContinuityPlan,
    envelope: &crate::history::data::CanonicalCommitEnvelopeBody,
) -> Result<(), TransactionCommitError> {
    crate::schema::validate_schema_continuity_body(envelope).map_err(|issue| {
        schema_continuity_conflict_from_issue(
            runtime,
            branch_id,
            envelope.schema_transition.as_ref(),
            issue,
            envelope,
        )
    })?;

    Ok(())
}
