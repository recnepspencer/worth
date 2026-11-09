//! Retain the actual executed no-effect candidate until the held lane applies it.
use super::*;
use crate::domain_computation::primary_graph::{
    output_lineage::{
        executed_adoption::PendingExecutedOutputAdoption, RetainedInputCutoffCandidate,
    },
    provider::WorthQueryApplicationBranchCommitCoordination,
};
use worth_relational::facade::mvcc::{
    PreparedRelationalCommitCandidate, RelationalCurrentOutputAdoption,
};

type Rebase = super::super::PreparedSourceFactRebase;
type Association = crate::domain_computation::primary_graph::output_lineage::required_publication::head::ReadmittedRequiredPublication;

type Claims = crate::domain_computation::primary_graph::PreparedPrerequisiteClaims;

/// This is not Ready or a publication. It owns the actual completed candidate,
/// original attempt and read custody until the coordinating owner qualifies it.
pub struct PreparedExecutedOutputAdoption {
    attempt: WorthQueryPrimaryGraphApplicationAttempt,
    _candidate: PreparedRelationalCommitCandidate,
    selected: RetainedInputCutoffCandidate,
    adoption: RelationalCurrentOutputAdoption,
    association: Association,
    before: precommit_snapshot::WorthQueryPrecommitSnapshot,
    rebase: Rebase,
    required: Claims,
    witness: crate::domain_computation::primary_graph::output_lineage::PreparedNativeOutputWitness,
}

pub(super) enum AdoptionPreparation {
    Declined {
        rebase: Rebase,
        required: Option<Claims>,
    },
    Selected {
        selected: RetainedInputCutoffCandidate,
        adoption: RelationalCurrentOutputAdoption,
        association: Association,
        rebase: Rebase,
        required: Claims,
    },
}

pub(super) fn prepare(
    execution: &worth_execution::ExecutionRequest<'_, '_>,
    provider: &WorthQueryPrimaryGraphProvider,
    attempt: &WorthQueryPrimaryGraphApplicationAttempt,
    candidate: &PreparedRelationalCommitCandidate,
    eligible: bool,
    rebase: Rebase,
    required: Option<Claims>,
) -> Result<AdoptionPreparation, crate::domain_computation::WorthQueryProviderSessionCommitStop> {
    let decline = |rebase, required| AdoptionPreparation::Declined { rebase, required };
    if !eligible || required.is_none() || attempt.has_nonrelational_completion_effects() {
        return Ok(decline(rebase, required));
    }
    let Some(selected) =
        crate::domain_computation::primary_graph::output_lineage::select_executed_adoption(
            &provider.graph.output_lineage,
            execution,
            attempt,
        )
        .map_err(pending)?
    else {
        return Ok(decline(rebase, required));
    };
    let Some(publisher) = provider.graph.application_publication.as_ref() else {
        return Ok(decline(rebase, required));
    };
    let head = selected
        .required_publication_head()
        .map_err(native_output_witness_stop)?;
    let association = provider.graph.with_runtime(execution, |runtime| {
        head.readmit_if_published(runtime, execution, &publisher.native,
            attempt.affinity().product_publication().observation().basis().relational_basis(),
            required.as_ref().expect("required claims checked").native_publication_family(),
            attempt.affinity().operation_scope().scope(),
            attempt.idempotency().source_partition_identity().expect("selected exact partition"),
            &provider.graph.layout)
    }).map_err(|stop| match stop {
        crate::domain_computation::primary_graph::output_lineage::required_publication::head::RequiredPublicationReadmissionStop::Selection(stop) => native_output_witness_stop(stop),
        crate::domain_computation::primary_graph::output_lineage::required_publication::head::RequiredPublicationReadmissionStop::Native(stop) => transaction_commit_stop(stop),
        crate::domain_computation::primary_graph::output_lineage::required_publication::head::RequiredPublicationReadmissionStop::Allocation(stop) => readmission_allocation(stop),
    })?;
    let Some(association) = association else {
        return Ok(decline(rebase, required));
    };
    let Some(adoption) = candidate
        .prepare_current_output_adoption(
            execution,
            attempt
                .affinity()
                .product_publication()
                .observation()
                .basis()
                .relational_basis(),
            association.publication(),
        )
        .map_err(transaction_commit_stop)?
    else {
        return Ok(decline(rebase, required));
    };
    Ok(AdoptionPreparation::Selected {
        selected,
        adoption,
        association,
        rebase,
        required: required.expect("selected only with genuine producer claims"),
    })
}

impl PreparedExecutedOutputAdoption {
    pub(super) fn new(
        attempt: WorthQueryPrimaryGraphApplicationAttempt,
        candidate: PreparedRelationalCommitCandidate,
        selected: RetainedInputCutoffCandidate,
        adoption: RelationalCurrentOutputAdoption,
        association: Association,
        before: precommit_snapshot::WorthQueryPrecommitSnapshot,
        rebase: Rebase,
        required: Claims,
        witness: crate::domain_computation::primary_graph::output_lineage::PreparedNativeOutputWitness,
    ) -> Self {
        Self {
            attempt,
            _candidate: candidate,
            selected,
            adoption,
            association,
            before,
            rebase,
            required,
            witness,
        }
    }

    pub(in crate::domain_computation::primary_graph) fn session(
        &self,
    ) -> &crate::domain_computation::provider_session::WorthQueryProviderSessionTerminalBinding
    {
        self.attempt.affinity().provider_session()
    }

    pub(in crate::domain_computation::primary_graph) fn publish(
        self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        provider: &WorthQueryPrimaryGraphProvider,
        coordination: &WorthQueryApplicationBranchCommitCoordination<'_>,
    ) -> Result<
        PendingExecutedOutputAdoption,
        crate::domain_computation::primary_graph::WorthQueryOutputDemandDenial,
    > {
        let Self {
            attempt,
            _candidate,
            selected,
            adoption,
            association,
            before,
            rebase,
            required,
            witness,
        } = self;
        let correspondence = attempt.seal_output_correspondence_from_adoption(&adoption);
        let (position, witness, facts) = provider.graph.with_runtime(execution, |runtime| {
            let position = runtime.read_truth().positioned_snapshot(before.as_snapshot())
                .map_err(|_| demand(crate::domain_computation::primary_graph::WorthQueryOutputDemandDenialKind::RetainedBasisUnavailable))?;
            let witness = witness.finish(&correspondence, runtime, before.as_snapshot())
                .ok_or_else(|| demand(crate::domain_computation::primary_graph::WorthQueryOutputDemandDenialKind::IncompleteDependencyCoverage))?;
            let facts = rebase.verify_executed_adoption(execution, runtime, before.as_snapshot(),
                &adoption, &correspondence,
                crate::domain_computation::primary_graph::application_attempt::retained_decision_facts::StorageControl::new(
                    execution.allocation_policy(), Some(attempt.affinity().publication_request())))
                .map_err(adoption_rebase_denial)?;
            Ok::<_, crate::domain_computation::primary_graph::WorthQueryOutputDemandDenial>((position, witness, facts))
        })?;
        crate::domain_computation::primary_graph::output_lineage::publish_executed_adoption(
            &provider.graph.output_lineage, &provider.graph.source_owner.invalidation_owner,
            attempt.affinity().product_publication().observation_port(), execution,
            &attempt, selected, &adoption, &association, &position, facts, witness, required, coordination)
            .map_err(|stop| match stop {
                crate::domain_computation::primary_graph::output_lineage::StablePublicationStop::Demand(stop) => stop,
                crate::domain_computation::primary_graph::output_lineage::StablePublicationStop::RequestAuthority(stop) =>
                    demand(crate::domain_computation::primary_graph::WorthQueryOutputDemandDenialKind::of_request_authorization(stop.kind())),
            })
    }
}

fn demand(
    kind: crate::domain_computation::primary_graph::WorthQueryOutputDemandDenialKind,
) -> crate::domain_computation::primary_graph::WorthQueryOutputDemandDenial {
    crate::domain_computation::primary_graph::WorthQueryOutputDemandDenial::new(
        kind,
        "executed output adoption",
    )
}
fn pending(
    denial: crate::domain_computation::primary_graph::WorthQueryOutputDemandDenial,
) -> crate::domain_computation::WorthQueryProviderSessionCommitStop {
    crate::domain_computation::WorthQueryProviderSessionCommitStop::Deferred(
        crate::domain_computation::WorthQueryProviderSessionCommitDeferred::required_prerequisite(
            denial,
            "executed output adoption qualification stopped",
        ),
    )
}
impl std::fmt::Debug for PreparedExecutedOutputAdoption {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("PreparedExecutedOutputAdoption")
            .field("session", self.session())
            .finish_non_exhaustive()
    }
}

fn adoption_rebase_denial(
    failure: super::super::RebaseVerificationFailure,
) -> crate::domain_computation::primary_graph::WorthQueryOutputDemandDenial {
    use crate::domain_computation::primary_graph::provider::RebaseVerificationReason as Reason;
    let stop = match failure.into_reason() {
        Reason::Execution(cause) => worth_relational::facade::mvcc::CompanionPreflightStop::Execution(cause),
        Reason::AdmissionDenied(stop) => stop,
        reason => return demand(crate::domain_computation::primary_graph::WorthQueryOutputDemandDenialKind::RetainedVerificationFactUnavailable(reason)),
    };
    crate::domain_computation::primary_graph::output_reuse::retained_fact_denial(
        crate::domain_computation::primary_graph::output_reuse::preflight_stop(stop),
        "executed adoption source verification",
    )
}
