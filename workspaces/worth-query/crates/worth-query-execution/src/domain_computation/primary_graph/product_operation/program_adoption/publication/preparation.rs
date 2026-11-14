//! Adoption uses the existing World Open -> Native preparation -> durable publication.
use crate::domain_computation::execution_runtime::product_world::{
    publication_denial::{
        WorthQueryProductPublicationDenial as RequestDenial,
        WorthQueryProductPublicationOutcome as Outcome,
    },
    WorthQueryPreparedProductPublication, WorthQueryProductPublicationBinding,
    WorthQueryProductPublicationRequestControl,
};
use crate::domain_computation::primary_graph::{
    program_occurrence::WorthQueryProgramSupportCustody,
    query_obligations::{QueryBranchAdoptionObligation, QueryObligationDenial},
    WorthQuerySelectedProductOperation,
};
use worth_foundational::facade::{
    CanonicalDigestId, DurableCompositeCommitIdentity, DurableCompositePublicationAttemptIdentity,
};
use worth_query_admission::facade::authenticated_principal::WorthQueryRequestScope;
use worth_query_installation::facade::ApplicationSchema;
use worth_relational::facade::mvcc::ValidatedRelationalProposal;
use worth_runtime_world::facade::*;
pub(in crate::domain_computation::primary_graph::product_operation::program_adoption) enum PreparedAdoptionPublication
{
    Memory(WorthQueryPreparedProductPublication),
    Home {
        publication: RuntimeWorldPublicationPort<(), (), (), (), ()>,
        prepared: WorldPreparedPublicationWithoutSignal,
        control: WorthQueryProductPublicationRequestControl,
        request: WorthQueryRequestScope,
    },
}
pub(in crate::domain_computation::primary_graph::product_operation::program_adoption) struct AdoptionRecordDetails
{
    pub source: CanonicalDigestId,
    pub target: CanonicalDigestId,
    pub selected: u64,
    pub migration: Option<(String, u64)>,
}
pub(in crate::domain_computation::primary_graph::product_operation::program_adoption) enum PreparationFailure
{
    Request(RequestDenial<NoEffectCompositePublication>),
    Obligation(QueryObligationDenial),
    Native(worth_relational::facade::transactions::TransactionCommitError),
    Opening(WorldOpenAttemptRefusal),
    Retained {
        effects: ProductUnpublishedOwnerEffects,
        opening: Option<WorldOpenAttemptRefusal>,
        native: Option<WorldNativePreparationDenial>,
    },
}
impl PreparedAdoptionPublication {
    pub(in crate::domain_computation::primary_graph::product_operation::program_adoption) fn prepare<
        Schema: ApplicationSchema,
    >(
        selected: &WorthQuerySelectedProductOperation<'_, Schema>,
        proposal: ValidatedRelationalProposal,
        request: &WorthQueryRequestScope,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        support: &mut WorthQueryProgramSupportCustody,
        details: AdoptionRecordDetails,
    ) -> Result<Self, PreparationFailure> {
        use PreparationFailure as Failure;
        let binding = selected.product().publication_binding();
        let home = binding
            .publication()
            .query_publication_home_binding(binding.observation())
            .map_err(Failure::Opening)?;
        let graph = &selected.application().runtime.primary_provider.graph;
        if home.is_none() {
            let candidate = graph
                .with_runtime(execution, |native| {
                    native.prepare_validated_proposal(execution, proposal)
                })
                .map_err(Failure::Native)?;
            return binding
                .prepare_relational_candidate(candidate, request, false)
                .map(Self::Memory)
                .map_err(Failure::Request);
        }
        let control = binding
            .request_control(request)
            .ok_or(Failure::Request(RequestDenial::ClockMismatch))?;
        let pending = binding
            .publication()
            .prepare_query_publication_without_signal(
                execution,
                binding.observation().clone(),
                binding
                    .admitted_program_source()
                    .map_err(Failure::Opening)?,
                worth_relational::facade::mvcc::RelationalTransactionIntent::ordinary(),
                control.cancellation(),
                control.deadline(),
            )
            .map_err(Failure::Opening)?;
        let planned = pending.planned_successor();
        let actual = planned.attempt_identity();
        let owner = actual.owner_identity().durable().expect("Home reservation");
        let expected = pending.expected_head();
        let commit = expected.selected_commit();
        let row = QueryBranchAdoptionObligation {
            attempt: DurableCompositePublicationAttemptIdentity::from_recorded(
                owner,
                actual.ordinal(),
            ),
            source: details.source,
            target: details.target,
            branch: expected.branch_identity().name().as_str().to_owned(),
            lifecycle: expected.lifecycle_incarnation().ordinal(),
            generation: planned.generation().get(),
            expected: DurableCompositeCommitIdentity::from_recorded(owner, commit.ordinal()),
            selected_entity_count: details.selected,
            migration: details.migration,
        };
        selected
            .application()
            .installed_program_support()
            .expect("adoption support")
            .lifecycle()
            .retain_branch_adoption(execution, row)
            .map_err(Failure::Obligation)?;
        support.transfer_to_durable();
        let opened = advance_open(pending, &binding, request, execution)?;
        let prepared = graph
            .with_runtime(execution, |native| {
                opened.prepare_native(native, execution, proposal)
            })
            .map_err(|failure| {
                let (open, cause) = failure.into_parts();
                Failure::Retained {
                    effects: open.retain(),
                    opening: None,
                    native: Some(cause),
                }
            })?;
        Ok(Self::Home {
            publication: binding.publication().clone(),
            prepared,
            control,
            request: request.clone(),
        })
    }
    pub(in crate::domain_computation::primary_graph::product_operation::program_adoption) fn durable_attempt(
        &self,
    ) -> Option<DurableCompositePublicationAttemptIdentity> {
        match self {
            Self::Memory(_) => None,
            Self::Home { prepared, .. } => {
                let actual = prepared.attempt_identity();
                Some(DurableCompositePublicationAttemptIdentity::from_recorded(
                    actual
                        .owner_identity()
                        .durable()
                        .expect("admitted Home preparation"),
                    actual.ordinal(),
                ))
            }
        }
    }
    pub(in crate::domain_computation::primary_graph::product_operation::program_adoption) fn planned_successor(
        &self,
    ) -> PlannedProductReferenceSuccessor {
        match self {
            Self::Memory(p) => p.planned_successor().clone(),
            Self::Home { prepared, .. } => prepared.planned_successor().clone(),
        }
    }
    pub(in crate::domain_computation::primary_graph::product_operation::program_adoption) fn expected_head(
        &self,
    ) -> &ProductBranchObservation {
        match self {
            Self::Memory(p) => p.expected_head(),
            Self::Home { prepared, .. } => prepared.expected_head(),
        }
    }
    pub(in crate::domain_computation::primary_graph::product_operation::program_adoption) fn execute(
        self,
        execution: worth_execution::ExecutionRequest<'_, '_>,
    ) -> Result<Outcome, RequestDenial<NoEffectCompositePublication>> {
        match self {
            Self::Memory(p) => p.execute(execution),
            Self::Home {
                publication,
                prepared,
                control,
                request,
            } => {
                let mut outcome = publication.execute_durable_without_signal(
                    execution,
                    prepared,
                    control.cancellation(),
                );
                loop {
                    match outcome {
                        WorldDurablePublicationOutcome::Performed(p) => {
                            return Ok(Outcome::Performed(p))
                        }
                        WorldDurablePublicationOutcome::Pending(pending) => {
                            if request.interruption().is_some() || execution.consult().is_err() {
                                return Ok(Outcome::ProductUnpublished(pending.retain()));
                            }
                            outcome = pending.advance(&execution);
                            std::thread::yield_now();
                        }
                        WorldDurablePublicationOutcome::ProductUnpublished(effects)
                        | WorldDurablePublicationOutcome::RecoveryRequired {
                            custody: effects,
                            ..
                        } => return Ok(Outcome::ProductUnpublished(effects)),
                    }
                }
            }
        }
    }
}
fn advance_open(
    mut pending: WorldPendingOpenAttempt,
    binding: &WorthQueryProductPublicationBinding,
    request: &WorthQueryRequestScope,
    execution: &worth_execution::ExecutionRequest<'_, '_>,
) -> Result<WorldOpenedPublicationAttempt, PreparationFailure> {
    let handle = pending.unpublished_recovery_handle();
    loop {
        match pending.advance(execution) {
            WorldOpenAttemptOutcome::Opened(opened) => return Ok(opened),
            WorldOpenAttemptOutcome::Pending(next) => {
                if request.interruption().is_some() || execution.consult().is_err() {
                    return Err(PreparationFailure::Retained {
                        effects: next.retain(),
                        opening: None,
                        native: None,
                    });
                }
                pending = next;
                std::thread::yield_now();
            }
            WorldOpenAttemptOutcome::Unresolved(next)
            | WorldOpenAttemptOutcome::EvidenceRequired(next) => {
                return Err(PreparationFailure::Retained {
                    effects: next.retain(),
                    opening: None,
                    native: None,
                })
            }
            WorldOpenAttemptOutcome::Refused(cause) => {
                return match binding.recovery().inspect_effects(&handle) {
                    Ok(effects) => Err(PreparationFailure::Retained {
                        effects,
                        opening: Some(cause),
                        native: None,
                    }),
                    Err(_) => Err(PreparationFailure::Opening(cause)),
                }
            }
        }
    }
}
