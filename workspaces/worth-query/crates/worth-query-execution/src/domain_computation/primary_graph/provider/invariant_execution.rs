use std::sync::Arc;

mod material;
mod receipt_closure;
mod relational_validation;
mod validation_control;
mod work_admission;
use material::{ApplicationInvariantCandidateMaterial, ApplicationInvariantSemanticMaterial};
use work_admission::admit_candidate_validator_work;

use super::invariant_execution_failure::{
    map_transaction_admission_failure, map_transaction_staging_failure, map_validation_failure,
};
use super::WorthQueryPrimaryGraphProvider;
use crate::domain_computation::{
    WorthQueryBoundInvariantExecutionView, WorthQueryInvariantExecutionDenialKind,
    WorthQueryInvariantExecutionFailure, WorthQueryInvariantExecutionProvider,
    WorthQueryInvariantProviderVerdict, WorthQueryInvariantStateLoadAdmission,
    WorthQueryInvariantStateLoadEvidence, WorthQueryInvariantStateLoadRequestView,
    WorthQueryInvariantStructuralCounters, WorthQueryInvariantVerdictAdmission,
    WorthQueryInvariantVerdictEvidence, WorthQueryProviderSessionView,
};

pub(super) struct WorthQueryInvariantWorkMint {
    _private: (),
}

impl WorthQueryInvariantExecutionProvider for Arc<WorthQueryPrimaryGraphProvider> {
    fn load_invariant_state(
        &self,
        session: WorthQueryProviderSessionView<'_>,
        request: WorthQueryInvariantStateLoadRequestView<'_>,
        admission: WorthQueryInvariantStateLoadAdmission,
    ) -> Result<WorthQueryInvariantStateLoadEvidence, WorthQueryInvariantExecutionFailure> {
        self.application_attempt_work.observe_invariant_state_load();
        let attempts = self
            .attempts
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        if !attempts.has_invariant_approved_candidate(session) {
            return Err(missing_candidate_failure());
        }
        let staged = attempts
            .staged_attempt(session)
            .ok_or_else(|| closure_failure("provider session has no staged application attempt"))?;
        let expected = ApplicationInvariantSemanticMaterial::from_staged(&staged)?.expected;
        if staged.expected_step_count() != staged.overlay_facts().len()
            || request.locators() != expected.as_slice()
        {
            return Err(closure_failure(
                "invariant state-load plan does not close over the proposed effects",
            ));
        }
        admission.admit(
            format!("primary-invariant-load:{}", staged.overlay_identity()),
            expected.clone(),
            WorthQueryInvariantStructuralCounters::new(
                expected.len(),
                expected.len() as u64,
                staged.overlay_facts().len() as u64,
            ),
        )
    }

    fn execute_invariant(
        &self,
        session: WorthQueryProviderSessionView<'_>,
        execution: WorthQueryBoundInvariantExecutionView<'_>,
        admission: WorthQueryInvariantVerdictAdmission,
    ) -> Result<WorthQueryInvariantProviderVerdict, WorthQueryInvariantExecutionFailure> {
        self.application_attempt_work.observe_invariant_execution();
        let material = self.semantic_invariant_material(session)?;
        let load_evidence = execution.state_load_evidence();
        material.validate_load(&execution, load_evidence)?;
        let attempts = self
            .attempts
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        let candidate = attempts
            .approved_candidate(session)
            .ok_or_else(missing_candidate_failure)?;
        let semantic_work = receipt_closure::requirement_work(
            candidate.invariant_evidence(),
            execution.requirement(),
            load_evidence.loaded_fact_locators().len(),
        )?;
        let evidence = WorthQueryInvariantVerdictEvidence::new(
            execution.requirement().slot(),
            "relational-installed-invariant-authority",
            load_evidence.identity(),
            semantic_work,
        )?;
        admission.passed(evidence)
    }
}

impl WorthQueryPrimaryGraphProvider {
    pub(in crate::domain_computation::primary_graph) fn admit_primary_candidate(
        &self,
        session: WorthQueryProviderSessionView<'_>,
        allocation_policy: worth_execution::ExecutionAllocationPolicy<'_, '_>,
    ) -> Result<(), WorthQueryInvariantExecutionFailure> {
        if self.take_skipped_invariant_owner_execution() {
            return Err(owner_failure());
        }
        let material = self.invariant_candidate_material(session)?;
        self.validate_and_retain_candidate(session, material, allocation_policy)
    }

    fn invariant_candidate_material(
        &self,
        session: WorthQueryProviderSessionView<'_>,
    ) -> Result<ApplicationInvariantCandidateMaterial, WorthQueryInvariantExecutionFailure> {
        let attempts = self
            .attempts
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        let staged = attempts
            .staged_attempt(session)
            .ok_or_else(|| closure_failure("invariant execution lost its staged attempt"))?;
        if staged.expected_step_count() != staged.overlay_facts().len() {
            return Err(closure_failure(
                "invariant execution lost proposed-effect closure",
            ));
        }
        ApplicationInvariantCandidateMaterial::from_staged(&staged)
    }

    fn semantic_invariant_material(
        &self,
        session: WorthQueryProviderSessionView<'_>,
    ) -> Result<ApplicationInvariantSemanticMaterial, WorthQueryInvariantExecutionFailure> {
        let attempts = self
            .attempts
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        let staged = attempts
            .staged_attempt(session)
            .ok_or_else(|| closure_failure("invariant execution lost its staged attempt"))?;
        if staged.expected_step_count() != staged.overlay_facts().len() {
            return Err(closure_failure(
                "invariant execution lost proposed-effect closure",
            ));
        }
        ApplicationInvariantSemanticMaterial::from_staged(&staged)
    }

    fn validate_and_retain_candidate(
        &self,
        session: WorthQueryProviderSessionView<'_>,
        material: ApplicationInvariantCandidateMaterial,
        allocation_policy: worth_execution::ExecutionAllocationPolicy<'_, '_>,
    ) -> Result<(), WorthQueryInvariantExecutionFailure> {
        let host_budget = self.resource_support.graph().envelope()
            .optional_scale_ceiling(worth_query_declaration::facade::domain_computation::WorthQuerySemanticScaleAxis::WorkItems);
        let semantic_work = admit_candidate_validator_work(&material, host_budget)?;
        let candidate = self.validate_relational_candidate(
            material.batch,
            &material.branch,
            &material.product,
            &material.request,
            &material.application_touches,
            material.aftermath_causality.as_ref(),
            allocation_policy,
        )?;
        let owner_work = receipt_closure::validate_receipt_closure(
            candidate.invariant_evidence(),
            &material.requirements,
            semantic_work,
        )?;
        super::super::product_operation::admit_required_invariants(
            candidate
                .invariant_evidence()
                .custom_invariant_execution_receipts(),
            material.producer_required_invariants,
        )
        .map_err(|_| {
            closure_failure("producer-required invariant did not pass before publication")
        })?;
        let touch_admission =
            super::application_touch_admission::admit_validated_application_touches(
                &candidate,
                &self.graph.layout,
                &material.application_graph_reads,
                &material.application_touches,
                &material.application_read_touch_overlap,
            )
            .map_err(|()| touch_failure())?;
        let summary = candidate.invariant_evidence().summary();
        let work = super::mutation_work::WorthQueryPrimaryMutationWorkCounters::new(
            WorthQueryInvariantWorkMint { _private: () },
            material.decision_facts,
            material.semantic.expected.len(),
            material.semantic.expected.len(),
            owner_work,
            summary.execution_count,
            summary.result_count,
            material.expected_step_preparation_work,
            touch_admission,
        );
        self.retain_validated_candidate(session, candidate, work)
    }

    fn retain_validated_candidate(
        &self,
        session: WorthQueryProviderSessionView<'_>,
        candidate: worth_relational::facade::mvcc::ValidatedRelationalProposal,
        work: super::mutation_work::WorthQueryPrimaryMutationWorkCounters,
    ) -> Result<(), WorthQueryInvariantExecutionFailure> {
        self.attempts
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
            .retain_invariant_approved(session, candidate, work)
            .map_err(closure_failure)
    }
}

fn validate_owner_evidence(
    evidence: &worth_relational::facade::mvcc::RelationalMutationInvariantEvidence,
    branch: &worth_relational::facade::history::BranchId,
) -> Result<(), WorthQueryInvariantExecutionFailure> {
    if evidence.branch() != branch {
        return Err(closure_failure(
            "Relational invariant evidence belongs to a different branch",
        ));
    }
    let summary = evidence.summary();
    (summary.execution_count == 3
        && summary.commit_boundary_seen
        && summary.mutation_sensitive_seen
        && summary.snapshot_publication_seen)
        .then_some(())
        .ok_or_else(owner_failure)
}

fn invariant_violation_probe() -> worth_relational::facade::transactions::MutationIntent {
    use worth_relational::facade::{identity, transactions};

    transactions::MutationIntent::Create(transactions::CreateIntent::Relation(
        transactions::RelationSpec {
            partition_id: identity::PartitionId::main(),
            kind_id: identity::KindId::new(u32::MAX),
            client_key: worth_relational::facade::symbols::ClientKey::raw(
                "invariant-mutation-probe",
            ),
            source: transactions::EntityReference::Existing(identity::EntityId::new(
                identity::PartitionId::main(),
                u64::MAX - 1,
                1,
            )),
            target: transactions::EntityReference::Existing(identity::EntityId::new(
                identity::PartitionId::main(),
                u64::MAX,
                1,
            )),
            fields: transactions::AspectFieldPatch::default(),
        },
    ))
}

#[cfg(test)]
fn undeclared_application_touch_probe(
    layout: &super::super::schema_layout::WorthQueryPrimaryGraphLayout,
    touches: &worth_query_installation::facade::WorthQueryOperationTouchContract,
) -> Result<
    worth_relational::facade::transactions::MutationIntent,
    WorthQueryInvariantExecutionFailure,
> {
    use worth_relational::facade::{identity::PartitionId, symbols::ClientKey, transactions};

    let kind = layout
        .application_entity_kind_without_create_scope(touches)
        .ok_or_else(touch_failure)?;
    Ok(transactions::MutationIntent::Create(
        transactions::CreateIntent::Entity(transactions::EntitySpec {
            partition_id: PartitionId::main(),
            kind_id: kind,
            client_key: ClientKey::raw("undeclared-application-touch-probe"),
            fields: transactions::AspectFieldPatch::default(),
        }),
    ))
}

fn closure_failure(detail: &'static str) -> WorthQueryInvariantExecutionFailure {
    WorthQueryInvariantExecutionFailure::new(
        WorthQueryInvariantExecutionDenialKind::StateLoadClosureMismatch,
        detail,
    )
}

fn missing_candidate_failure() -> WorthQueryInvariantExecutionFailure {
    closure_failure("semantic invariant execution requires an owner-sealed candidate")
}

fn owner_failure() -> WorthQueryInvariantExecutionFailure {
    WorthQueryInvariantExecutionFailure::new(
        WorthQueryInvariantExecutionDenialKind::ProviderRejected,
        "Relational rejected the installed proposed-state invariant",
    )
}

fn touch_failure() -> WorthQueryInvariantExecutionFailure {
    WorthQueryInvariantExecutionFailure::new(
        WorthQueryInvariantExecutionDenialKind::ProviderRejected,
        "Relational validated candidate touches exceed the installed application contract",
    )
}

fn aftermath_failure() -> WorthQueryInvariantExecutionFailure {
    WorthQueryInvariantExecutionFailure::new(
        WorthQueryInvariantExecutionDenialKind::ProviderRejected,
        "the admitted aftermath parent is no longer the current Relational head",
    )
}
