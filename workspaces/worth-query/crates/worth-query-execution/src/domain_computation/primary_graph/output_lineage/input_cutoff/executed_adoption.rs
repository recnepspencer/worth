//! Publish fresh executed support through the existing stable cutover owner.
use super::super::StablePublicationStop;
use super::super::{
    executed_adoption::PendingExecutedOutputAdoption, invalidation::SourceInvalidationOwner,
    SealedNativeOutputWitness, WorthQueryApplicationOutputLineage,
};
use super::{stable_publication::prepare_address, RetainedInputCutoffCandidate};
use crate::domain_computation::primary_graph::{
    application_checkpoint::facts::FactPayload,
    provider::{
        WorthQueryApplicationBranchCommitCoordination, WorthQueryPrimaryGraphApplicationAttempt,
    },
    PreparedPrerequisiteClaims, WorthQueryOutputDemandDenial,
    WorthQueryOutputDemandDenialKind as Kind,
};
use std::sync::{Arc, Mutex, OnceLock};

pub(in crate::domain_computation::primary_graph) fn select(
    owner: &Arc<Mutex<WorthQueryApplicationOutputLineage>>,
    execution: &worth_execution::ExecutionRequest<'_, '_>,
    attempt: &WorthQueryPrimaryGraphApplicationAttempt,
) -> Result<Option<RetainedInputCutoffCandidate>, WorthQueryOutputDemandDenial> {
    let Some(selection) = ExecutedAdoptionSelection::from_attempt(attempt) else {
        return Ok(None);
    };
    selection.select(owner, execution)
}

/// Owned selectors captured without taking the lineage or Native locks while
/// the provider's attempt store is borrowed. They confer no adoption authority.
pub(in crate::domain_computation::primary_graph) struct ExecutedAdoptionSelection {
    binding: crate::equality_type_id::EqualityTypeId,
    partition: [u8; 32],
    scope: crate::domain_computation::authorization::WorthQueryOperationScopeBinding,
    product: crate::domain_computation::execution_runtime::product_world::WorthQueryProductPublicationBinding,
}
impl ExecutedAdoptionSelection {
    pub(in crate::domain_computation::primary_graph) fn native_branch_id(
        &self,
    ) -> &worth_relational::facade::history::BranchId {
        self.product
            .observation()
            .basis()
            .relational_basis()
            .descriptor()
            .branch_id()
    }
    pub(in crate::domain_computation::primary_graph) fn from_attempt(
        attempt: &WorthQueryPrimaryGraphApplicationAttempt,
    ) -> Option<Self> {
        Some(Self {
            binding: attempt.output_binding_type()?,
            partition: attempt.idempotency().source_partition_identity()?,
            scope: attempt.affinity().operation_scope().clone(),
            product: attempt.affinity().product_publication().clone(),
        })
    }
    pub(in crate::domain_computation::primary_graph) fn select(
        &self,
        owner: &Arc<Mutex<WorthQueryApplicationOutputLineage>>,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
    ) -> Result<Option<RetainedInputCutoffCandidate>, WorthQueryOutputDemandDenial> {
        self.select_native(owner, execution).map_err(|stop| {
            crate::domain_computation::primary_graph::output_reuse::retained_fact_denial(
                crate::domain_computation::primary_graph::output_reuse::preflight_stop(stop),
                "executed adoption lineage selection",
            )
        })
    }
    pub(in crate::domain_computation::primary_graph) fn select_native(
        &self,
        owner: &Arc<Mutex<WorthQueryApplicationOutputLineage>>,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
    ) -> Result<
        Option<RetainedInputCutoffCandidate>,
        worth_relational::facade::mvcc::CompanionPreflightStop,
    > {
        owner
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .prior_input_cutoff_candidate_for_binding(
                self.binding,
                &self.scope,
                self.product.observation(),
                self.partition,
                execution,
            )
    }
}

pub(in crate::domain_computation::primary_graph) fn publish(
    owner: &Arc<Mutex<WorthQueryApplicationOutputLineage>>,
    source_owner: &SourceInvalidationOwner,
    world: &worth_runtime_world::facade::RuntimeWorldObservationPort,
    execution: &worth_execution::ExecutionRequest<'_, '_>,
    attempt: &WorthQueryPrimaryGraphApplicationAttempt,
    candidate: RetainedInputCutoffCandidate,
    adoption: &worth_relational::facade::mvcc::RelationalCurrentOutputAdoption,
    association: &super::super::required_publication::head::ReadmittedRequiredPublication,
    selected: &worth_relational::facade::runtime::PositionedRelationalSnapshot,
    facts: FactPayload,
    witness: Arc<OnceLock<SealedNativeOutputWitness>>,
    mut required: PreparedPrerequisiteClaims,
    coordination: &WorthQueryApplicationBranchCommitCoordination<'_>,
) -> Result<PendingExecutedOutputAdoption, StablePublicationStop> {
    let observation = attempt.affinity().product_publication().observation();
    if adoption.basis() != observation.basis().relational_basis().descriptor()
        || selected.root_id() != adoption.basis().root_identity()
    {
        return Err(denial(Kind::PublicationStale).into());
    }
    let binding = attempt
        .output_binding_type()
        .ok_or_else(|| denial(Kind::ForeignSource))?;
    let address = prepare_address(
        owner,
        candidate,
        selected,
        attempt.affinity().operation_scope(),
        binding,
        observation,
        coordination,
        execution,
    )?;
    let publication = address.prepare_executed_record(
        adoption,
        association,
        attempt,
        facts,
        witness,
        &mut required,
        execution,
    )?;
    required.reserve_identity(publication.identity())?;
    let registration = publication
        .prepare_executed_registration(source_owner, adoption, required.work_membership())
        .map_err(|stop| {
            use super::super::invalidation::SettlementRegistrationStop as Stop;
            let cause = match stop {
                Stop::Admission(stop) => {
                    crate::domain_computation::primary_graph::output_reuse::retained_fact_denial(
                        crate::domain_computation::primary_graph::output_reuse::preflight_stop(
                            stop,
                        ),
                        "executed adoption current registration",
                    )
                }
                Stop::Foreign => denial(Kind::ForeignSource),
                Stop::SourceUnavailable | Stop::Alignment(_) => {
                    denial(Kind::RetainedBasisUnavailable)
                }
                Stop::Edit(_) => denial(Kind::PublicationStale),
            };
            cause.with_subject(format!(
                "{} source {:?}: executed registration {stop:?}",
                adoption.meaning().family,
                adoption.meaning().source_scope
            ))
        })?;
    let displaced = publication.displaced_settlement();
    let publication = publication.publish(
        attempt.affinity().admitted_request(),
        attempt.affinity().operation(),
        observation,
        world,
        registration,
        execution,
    )?;
    required
        .publish(Arc::clone(publication.exact_settlement()))
        .retire(displaced, source_owner);
    Ok(PendingExecutedOutputAdoption {
        publication,
        session: attempt.affinity().provider_session().clone(),
    })
}

fn denial(kind: Kind) -> WorthQueryOutputDemandDenial {
    WorthQueryOutputDemandDenial::new(kind, "executed output adoption")
}
