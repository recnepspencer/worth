//! One Native candidate owns the declared authored stages and required producer wave.
use super::producer::execution::{CoupledSourceReadStop, PreparedCoupledWriter};
use super::producer::{
    execution, registry, synchronous_outputs, WorthQueryOutputDemandDenial,
    WorthQueryOutputDemandDenialKind,
};
use worth_query_execution_attempt::facade::ports::PrimaryGraphProviderInternalPort as _;
use worth_query_execution_lineage::facade::ports::CleanupRequiredInternalPort as _;
use worth_query_execution_runtime::facade::ports::ProductBranchLeaseInternalPort as _;
use worth_query_installation::facade::ApplicationSchema;
mod applied_outputs;
mod authored_stages;
mod candidate_reads;
mod disposal;
mod input_preparation;
mod ledger;
mod preparation;
mod qualification;
pub(in crate::domain_computation::primary_graph::application_contribution) use qualification::ReopenTarget;
mod settlement;
mod source_alignment;
mod source_selection;
mod succession;
mod verified_parents;
pub use input_preparation::{PlannedOutput, PreparedCoupledInputs};
pub use source_selection::SelectedCoupledSource;
pub(in crate::domain_computation::primary_graph::application_contribution) use qualification::AppendedMember;
mod validation;
use crate::domain_computation::primary_graph::{
    application_attempt::coupled_outputs::DeclaredCoupledOutputDemands, WorthQueryAdvancementPhase,
    WorthQueryPrimaryGraphApplicationRuntime,
};
use std::cell::RefCell;
use std::ops::ControlFlow;
use worth_query_admission::facade::authenticated_principal::{
    WorthQueryAuthenticatedExternalPrincipal, WorthQueryRequestScope,
};
use worth_relational::facade::mvcc::{
    RelationalAuthoredCoupledCandidate, RelationalAuthoredCoupledProducerAssembly,
    RelationalCoupledAssemblyFailure,
};

pub struct CoupledApplicationAssembly<
    'a,
    'run,
    'phase,
    S: ApplicationSchema,
> {
    phase: &'run WorthQueryAdvancementPhase<'phase>,
    coordination: &'a crate::domain_computation::primary_graph::provider::WorthQueryApplicationBranchCommitCoordination<'a>,
    pub application: &'a WorthQueryPrimaryGraphApplicationRuntime<S>,
    pub outer_application: Option<&'a (dyn std::any::Any + Send + Sync)>,
    pub principal: Option<&'a WorthQueryAuthenticatedExternalPrincipal<S>>,
    pub before: &'a crate::basis::WorthQueryProductBranchLease,
    pub request: &'a WorthQueryRequestScope,
    demands: &'a DeclaredCoupledOutputDemands,
    root_session: crate::domain_computation::provider_session::WorthQueryProviderSessionTerminalBinding,
    rounds: RefCell<synchronous_outputs::rounds::limits::ProducerRoundBudget>,
    writers: RefCell<Vec<CoupledWriter<'run, S>>>,
    authored: RefCell<Vec<AuthoredWriter<'run, S>>>,
    denial: RefCell<Option<WorthQueryOutputDemandDenial>>,
    abort_failures: RefCell<Vec<crate::domain_computation::primary_graph::WorthQueryApplicationUncommitted>>,
    /// The last assembly round ended the edit (`Break`).
    broke: std::cell::Cell<bool>,
    /// The demand that owes the work now being prepared.
    owing: RefCell<OwedDemand>,
    /// Parent outputs identified so far; the next one's read identity.
    parent_reads: std::cell::Cell<usize>,
    /// The parent outputs the route now being prepared consumes.
    consuming: RefCell<Vec<crate::domain_computation::primary_graph::provider::ParentRead>>,
    /// Retained derivations and which later appends made them suspect (R2).
    ledger: RefCell<ledger::DerivationLedger>,
    /// The route now being read derives again: no prepared or verified reuse.
    superseding: std::cell::Cell<bool>,
    /// The wave task whose discovery is now retaining support (its owner).
    discovering: std::cell::Cell<Option<(usize, bool)>>,
    /// Parent outputs some derivation consumed (R3): (parent read, consumer read).
    consumed: RefCell<worth_execution::ExecutionOrderedMap<(usize, usize), ()>>,
    /// Retentions superseded at their authenticated invalidator, to reopen.
    reopens: RefCell<Vec<qualification::Reopen>>,
    /// Aligned managed Source reads, marked by each later append (#310).
    alignments: RefCell<source_alignment::Alignments>,
    /// Name reads a decision took on the candidate (#361, #426).
    candidate_reads: RefCell<candidate_reads::CandidateReads>,
    /// Successor rounds appended: their writers took their chain's slot.
    successors: std::cell::Cell<usize>,
}
/// The applied position of the earliest stage whose effect demands some work.
/// Work demanded through it (prerequisites, discovered children, continuations
/// and the writers they prepare) shares the same cell. A route that an earlier
/// demand also reaches lowers it to that demand, so the position is a lower
/// bound: settlement by position can only refuse more, never accept a stale read.
#[derive(Clone)]
pub struct OwedDemand(std::rc::Rc<std::cell::Cell<usize>>);
impl OwedDemand {
    pub fn at(position: usize) -> Self {
        Self(std::rc::Rc::new(std::cell::Cell::new(position)))
    }
    pub fn position(&self) -> usize {
        self.0.get()
    }
    /// Coalesce another demand for the same route: the earliest one owes it.
    pub fn coalesce(&self, other: &Self) {
        self.0.set(self.position().min(other.position()));
    }
}
struct AuthoredWriter<'run, S> {
    capture: crate::domain_computation::primary_graph::application_contribution::authored_edit::CapturedAuthoredEditStage<S>,
    physical: usize,
    position: usize,
    stage: Option<CoupledWriterStage<'run>>,
}
struct CoupledWriter<'run, S> {
    position: usize,
    owed: OwedDemand,
    /// The parent outputs this route's derivation consumes.
    consumes: Vec<crate::domain_computation::primary_graph::provider::ParentRead>,
    /// This route's own derived output, as its consumers carry it.
    produces: crate::domain_computation::primary_graph::provider::ParentRead,
    producer: std::sync::Arc<registry::InstalledProducerProvider<S>>,
    stage: Option<CoupledWriterStage<'run>>,
    meaning: worth_relational::facade::mvcc::RequiredOutputPublicationMeaning,
    /// For a successor round: its chain head's original idempotency key,
    /// which every later round re-creates in place (§1.5 Identity).
    head_key: Option<String>,
}
enum CoupledWriterStage<'run> {
    Inspected(
        crate::domain_computation::primary_graph::application_attempt::InspectedCoupledWriter<'run>,
    ),
    Prepared(
        crate::domain_computation::primary_graph::application_attempt::CommitPreparedCoupledWriter<
            'run,
        >,
    ),
}
impl<'a, 'run, 'phase, S: ApplicationSchema> CoupledApplicationAssembly<'a, 'run, 'phase, S> {
    /// Every member append, successor rounds included, takes the next one.
    fn applied_position(&self) -> usize {
        self.authored.borrow().len() + self.writers.borrow().len() + self.successors.get()
    }
    pub fn new(
        phase: &'run WorthQueryAdvancementPhase<'phase>,
        coordination: &'a crate::domain_computation::primary_graph::provider::WorthQueryApplicationBranchCommitCoordination<'a>,
        application: &'a WorthQueryPrimaryGraphApplicationRuntime<S>,
        outer_application: Option<&'a (dyn std::any::Any + Send + Sync)>,
        principal: Option<&'a WorthQueryAuthenticatedExternalPrincipal<S>>,
        before: &'a crate::basis::WorthQueryProductBranchLease,
        request: &'a WorthQueryRequestScope,
        demands: &'a DeclaredCoupledOutputDemands,
        root_session: crate::domain_computation::provider_session::WorthQueryProviderSessionTerminalBinding,
    ) -> Self {
        Self {
            phase,
            coordination,
            application,
            outer_application,
            principal,
            before,
            request,
            demands,
            root_session,
            rounds: RefCell::new(
                synchronous_outputs::rounds::limits::ProducerRoundBudget::new(demands.round_limits),
            ),
            writers: RefCell::new(Vec::new()),
            authored: RefCell::new(Vec::new()),
            denial: RefCell::new(None),
            abort_failures: RefCell::new(Vec::new()),
            broke: std::cell::Cell::new(false),
            owing: RefCell::new(OwedDemand::at(0)),
            parent_reads: std::cell::Cell::new(0),
            consuming: RefCell::new(Vec::new()),
            ledger: RefCell::new(ledger::DerivationLedger::default()),
            superseding: std::cell::Cell::new(false),
            discovering: std::cell::Cell::new(None),
            consumed: RefCell::new(worth_execution::ExecutionOrderedMap::default()),
            reopens: RefCell::new(Vec::new()),
            alignments: RefCell::default(),
            candidate_reads: RefCell::default(),
            successors: std::cell::Cell::new(0),
        }
    }
    pub fn owing(&self) -> OwedDemand {
        self.owing.borrow().clone()
    }
    /// The work prepared next is owed by this demand.
    pub fn owe(&self, demand: &OwedDemand) {
        *self.owing.borrow_mut() = demand.clone();
    }
    /// The work prepared next is owed by a new demand at this position.
    pub fn owe_at(&self, position: usize) {
        *self.owing.borrow_mut() = OwedDemand::at(position);
    }
    pub fn retain_discovery_support(
        &self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        source: worth_relational::facade::identity::EntityId,
        support: crate::domain_computation::primary_graph::provider::CandidateReadSupport,
    ) -> Result<(), WorthQueryOutputDemandDenial> {
        self.application
            .runtime
            .primary_provider
            .retain_candidate_read_support(
                execution,
                &self.root_session,
                source,
                self.applied_position(),
                self.discovering.get(),
                support,
            )
            .map_err(execution::denial::candidate_support_denied)
    }
    /// The wave tasks whose outgoing support is equal at `current` (PR C §4).
    pub fn refresh_outgoing_support(
        &self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        current: &worth_relational::facade::runtime::RelationalPinnedRead<
            worth_relational::facade::runtime::RelationalAppliedCandidateReadIdentity,
        >,
    ) -> Result<Vec<usize>, WorthQueryOutputDemandDenial> {
        self.application
            .runtime
            .primary_provider
            .refresh_outgoing_read_support(execution, &self.root_session, self.before, current)
            .map_err(execution::denial::candidate_support_denied)
    }
    pub fn take_denial(&self) -> Option<WorthQueryOutputDemandDenial> {
        self.denial.borrow_mut().take()
    }
    /// The last round ended the edit and left neither a refusing subject nor
    /// an abort failure for the caller: the refusal was lost.
    pub fn broke_without_subject(&self) -> bool {
        self.broke.get()
            && self.denial.borrow().is_none()
            && self.abort_failures.borrow().is_empty()
    }
    pub fn settle_writers(
        &self, execution: &worth_execution::ExecutionRequest<'_, '_>,
        root: crate::domain_computation::primary_graph::application_attempt::WorthQueryProviderProgressionOutcome,
    ) -> crate::domain_computation::primary_graph::application_attempt::WorthQueryProviderProgressionOutcome{
        use crate::domain_computation::primary_graph::application_attempt::WorthQueryProviderProgressionOutcome as Progression;
        use crate::domain_computation::primary_graph::WorthQueryApplicationUncommitted as Uncommitted;
        let unpublished = match &root {
            Progression::ProductUnpublished(value) => Some(value),
            _ => None,
        };
        let mut failure = self.settle_authored(execution, unpublished);
        for writer in self.writers.borrow_mut().drain(..) {
            let Some(CoupledWriterStage::Prepared(prepared)) = writer.stage else {
                unreachable!("whole candidate prepares each writer before World")
            };
            let completion = prepared.completion();
            let provider = &self.application.runtime.primary_provider;
            let effected = unpublished.is_some()
                || prepared.has_original_completion()
                || provider
                    .inspect_pending_application_idempotency(
                        self.before.observation().lifecycle_incarnation(),
                    )
                    .is_some();
            let outcome = if effected {
                writer.producer.coupled_executor().complete_coupled_writer(
                    execution,
                    self.phase,
                    self.application,
                    prepared,
                    unpublished,
                )
            } else {
                self.retain_abort_outcome(prepared.abort(execution, self.phase, self.application));
                continue;
            };
            let uncommitted = match outcome.landed() {
                Ok(landed) => {
                    if let crate::domain_computation::primary_graph::application_attempt::WorthQueryApplicationLandedCommit::Live { receipt } = &landed {
                        execution::contact_observation::record_live_completion(
                            self.application, execution, &writer.producer.declaration.identity,
                            completion.source().root(), receipt, self.before,
                            match &root { Progression::Committed(receipt) => Some(receipt.original_publication()), _ => None },
                        );
                    }
                    completion.complete(landed);
                    continue;
                }
                Err(Uncommitted::ProductUnpublished(_)) => continue,
                Err(failure) => failure,
            };
            failure = Some(match (failure.take(), uncommitted) {
                (Some(prior), Uncommitted::CleanupRequired(cleanup)) => {
                    let (_, cleanup) = cleanup.into_parts();
                    Uncommitted::CleanupRequired(crate::domain_computation::primary_graph::application_attempt::WorthQueryApplicationCleanupRequired::from_parts(prior.into(), cleanup))
                }
                (Some(prior), _) => prior,
                (None, failure) => failure,
            });
        }
        self.finish_abort(failure.map(Progression::CoupledFailure).unwrap_or(root))
    }
    pub fn authorize_members(
        &self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        coordination: &crate::domain_computation::primary_graph::provider::WorthQueryApplicationBranchCommitCoordination<'_>,
    ) -> Result<(), crate::domain_computation::authorization::WorthQueryOperationAuthorizationDenial>
    {
        for writer in self.authored.borrow().iter() {
            let Some(CoupledWriterStage::Prepared(prepared)) = writer.stage.as_ref() else {
                unreachable!("validated authored stage")
            };
            writer.capture.executor.authorize(
                prepared,
                execution,
                self.application,
                coordination,
            )?;
        }
        for writer in self.writers.borrow().iter() {
            let Some(CoupledWriterStage::Prepared(approved)) = writer.stage.as_ref() else {
                unreachable!("validated group contains approved writers")
            };
            writer
                .producer
                .coupled_executor()
                .authorize_coupled_writer(execution, self.application, approved, coordination)?;
        }
        Ok(())
    }
}
impl<S: ApplicationSchema> RelationalAuthoredCoupledProducerAssembly
    for CoupledApplicationAssembly<'_, '_, '_, S>
{
    fn assemble(
        &self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        candidate: &mut RelationalAuthoredCoupledCandidate<'_>,
    ) -> Result<ControlFlow<()>, RelationalCoupledAssemblyFailure> {
        // Native may abandon a candidate to load one of its 256 symbol buckets
        // and call this logical envelope again (at most 257 attempts). A real
        // refusal belongs to the envelope, so no later callback may erase it.
        if self.denial.borrow().is_some() {
            self.broke.set(true);
            return Ok(ControlFlow::Break(()));
        }
        self.broke.set(false);
        // Every round, including a cold-symbol retry, starts at the prefix;
        // no position survives from an abandoned candidate.
        self.owe_at(0);
        if !self.abort_writers(execution) {
            self.broke.set(true);
            return Ok(ControlFlow::Break(()));
        }
        if self
            .application
            .runtime
            .primary_provider
            .application_attempt_operation_control_ref()
            .take_break_without_subject()
        {
            self.broke.set(true);
            return Ok(ControlFlow::Break(()));
        }
        // A cold-symbol retry abandoned the complete candidate without an
        // effect (G4/D1619). Route rounds are per candidate, while the original
        // execution request still pays all work and Native bounds retries by
        // its 256 buckets. Cold placement cannot consume a logical route round.
        *self.rounds.borrow_mut() = synchronous_outputs::rounds::limits::ProducerRoundBudget::new(
            self.demands.round_limits,
        );
        *self.ledger.borrow_mut() = ledger::DerivationLedger::default();
        *self.consumed.borrow_mut() = worth_execution::ExecutionOrderedMap::default();
        self.reopens.borrow_mut().clear();
        *self.alignments.borrow_mut() = Default::default();
        *self.candidate_reads.borrow_mut() = Default::default();
        self.successors.set(0);
        self.superseding.set(false);
        let prepared = self
            .application
            .runtime
            .primary_provider
            .reset_candidate_read_support(&self.root_session)
            .map_err(|cause| {
                CoupledSourceReadStop::Query(
                    execution::denial::candidate_support_denied(cause).into(),
                )
            })
            .and_then(|()| self.prepare(execution, candidate));
        match prepared {
            Ok(()) => Ok(ControlFlow::Continue(())),
            Err(CoupledSourceReadStop::Native(cause)) => Err(cause),
            Err(CoupledSourceReadStop::Query(cause)) => {
                self.denial
                    .borrow_mut()
                    .get_or_insert_with(|| cause.into_output_denial());
                self.broke.set(true);
                Ok(ControlFlow::Break(()))
            }
        }
    }
}
