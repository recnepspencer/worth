mod publication_appendix;
use std::sync::Arc;

mod authored_coupling;
mod authored_restoration;
mod candidate_admission;
mod material;
mod receipt_closure;
use candidate_admission::CandidateInvariantJudgment;
pub(in crate::domain_computation::primary_graph) use receipt_closure::ClosedCoupledInvariantReceipts;
mod relational_validation;
mod validation_control;
pub(in crate::domain_computation::primary_graph) use authored_coupling::CoupledProducerAssembly;
use material::{ApplicationInvariantCandidateMaterial, ApplicationInvariantSemanticMaterial};

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
        if attempts.approved_invariant_evidence(session).is_none() {
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
        let evidence = attempts
            .approved_invariant_evidence(session)
            .ok_or_else(missing_candidate_failure)?;
        let semantic_work = receipt_closure::requirement_work(
            evidence,
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
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        session: WorthQueryProviderSessionView<'_>,
        allocation_policy: worth_execution::ExecutionAllocationPolicy<'_, '_>,
        allow_adoption: bool,
    ) -> Result<(), WorthQueryInvariantExecutionFailure> {
        if self.take_skipped_invariant_owner_execution() {
            return Err(owner_failure());
        }
        // This controls receipt omission, not write authority. History and
        // coupled candidates retain the receipt because their commit owner
        // does not have the ordinary executed-adoption posture.
        let material = self.invariant_candidate_material(
            session,
            allow_adoption.then_some((execution, allocation_policy)),
        )?;
        self.validate_and_retain_candidate(execution, session, material, None, allocation_policy)
    }

    pub(in crate::domain_computation::primary_graph) fn admit_primary_candidate_coupled(
        &self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        session: WorthQueryProviderSessionView<'_>,
        assembly: &dyn CoupledProducerAssembly,
        allocation_policy: worth_execution::ExecutionAllocationPolicy<'_, '_>,
    ) -> Result<(), WorthQueryInvariantExecutionFailure> {
        if self.take_skipped_invariant_owner_execution() {
            return Err(owner_failure());
        }
        let material = self.invariant_candidate_material(session, None)?;
        self.validate_and_retain_candidate(
            execution,
            session,
            material,
            Some(assembly),
            allocation_policy,
        )
    }

    fn invariant_candidate_material(
        &self,
        session: WorthQueryProviderSessionView<'_>,
        appendix_policy: Option<(
            &worth_execution::ExecutionRequest<'_, '_>,
            worth_execution::ExecutionAllocationPolicy<'_, '_>,
        )>,
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
        ApplicationInvariantCandidateMaterial::from_staged(
            &staged,
            &self.graph.layout,
            appendix_policy,
        )
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
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        session: WorthQueryProviderSessionView<'_>,
        material: ApplicationInvariantCandidateMaterial,
        assembly: Option<&dyn CoupledProducerAssembly>,
        allocation_policy: worth_execution::ExecutionAllocationPolicy<'_, '_>,
    ) -> Result<(), WorthQueryInvariantExecutionFailure> {
        let mut material = self.prepare_authored_restoration(execution, material)?;

        let native_assembly = assembly.map(|assembly| assembly as &dyn worth_relational::facade::mvcc::RelationalAuthoredCoupledProducerAssembly);
        let coupling =
            self.native_authored_coupling(execution, session, &mut material, native_assembly)?;
        self.bind_independent_restoration(&mut material)?;
        let appendix = if assembly.is_none()
            && coupling.is_none()
            && material.restoration_publication.is_none()
        {
            publication_appendix::select(self, execution, &mut material)?
        } else {
            None
        };
        let (candidate, predicate) = self.validate_relational_candidate(
            execution,
            material.batch.clone(),
            appendix,
            &material.content_attachments,
            &material.branch,
            &material.product,
            &material.request,
            &material.application_touches,
            material.aftermath_causality.as_ref(),
            material.restoration_publication.take(),
            coupling,
            allocation_policy,
        )?;
        let closed = if let Some(assembly) = assembly {
            let mut requirements = material.requirements.clone();
            requirements.extend(assembly.invariant_requirements(execution, self, &candidate)?);
            Some(ClosedCoupledInvariantReceipts::admit(
                &candidate,
                &requirements,
            )?)
        } else {
            None
        };
        let judgment = match closed.as_ref() {
            Some(receipts) => CandidateInvariantJudgment::GroupOwner(receipts),
            None => CandidateInvariantJudgment::Standalone(&candidate),
        };
        let work = self.admit_candidate_material(judgment, &material, || {
            super::application_touch_admission::admit_validated_application_touches(
                execution,
                &candidate,
                &self.graph.layout,
                &material.application_graph_reads,
                &material.application_touches,
                &material.application_read_touch_overlap,
            )
            .map_err(
                super::application_touch_admission::TouchAdmissionFailure::into_invariant_failure,
            )
        })?;
        if let Some(assembly) = assembly {
            assembly.admit_validated(
                execution,
                self,
                closed.as_ref().expect("actual coupled closure"),
            )?;
        }
        let members = assembly.map_or_else(Vec::new, CoupledProducerAssembly::prepared_members);
        super::coupled_members::require_complete(execution, &candidate, &members)?;
        self.retain_validated_candidate(session, candidate, work, predicate, members)
    }

    fn retain_validated_candidate(
        &self,
        session: WorthQueryProviderSessionView<'_>,
        candidate: worth_relational::facade::mvcc::ValidatedRelationalProposal,
        work: super::mutation_work::WorthQueryPrimaryMutationWorkCounters,
        predicate: Option<crate::domain_computation::primary_graph::application_query::derived_view::PredicatePreparationOwner>,
        members: Vec<super::CoupledApplicationMember>,
    ) -> Result<(), WorthQueryInvariantExecutionFailure> {
        let mut incoming = Some(
            super::application_attempt_state::WorthQueryApprovedApplicationCandidate {
                members,
                native: candidate,
                work,
                predicate,
            },
        );
        let result = self
            .attempts
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
            .retain_invariant_approved(session, &mut incoming);
        drop(incoming); // Rejected incoming custody retires after the attempt gate.
        result.map_err(closure_failure)
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
    (summary.commit_boundary_seen
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

pub(super) fn closure_failure(detail: &'static str) -> WorthQueryInvariantExecutionFailure {
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

pub(super) fn touch_failure() -> WorthQueryInvariantExecutionFailure {
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
