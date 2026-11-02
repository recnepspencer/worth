//! Pin real Live history before the read; join and register only its actual image.
use super::*;
use crate::domain_computation::primary_graph::application_query::derived_view::PredicateWorld;
use crate::domain_computation::primary_graph::live_delivery::WorthQueryLiveAdmissionDenial;

impl<S: ApplicationSchema> WorthQueryPrimaryGraphApplicationRuntime<S> {
    pub fn open_application_managed_index_watch<B, I>(
        &self,
        phase: &WorthQueryAdvancementPhase<'_>,
        binding: &WorthQueryInstalledApplicationQueryBinding<S, B>,
        external: &WorthQueryAuthenticatedExternalPrincipal<S>,
        mut plan: BoundPlan<'_, S, B>,
    ) -> Result<
        (
            WorthQueryApplicationOneShotResult<B::Query, ResultValue<S, B>>,
            WorthQueryManagedIndexWatch<S, I>,
        ),
        Denial,
    >
    where
        B: ApplicationQueryBinding<S>,
        I: ApplicationManagedIndexBinding<S>,
        ResultValue<S, B>: WorthQueryApplicationProjection<S, B::Query>,
    {
        let execution = phase
            .execution_request_for(&self.product_runtime)
            .map_err(|_| Denial::new(ReceiverCause::Refused(Refusal::ForeignPhase), None))?;
        let allocation = SourceFactsAllocation::open(&execution)
            .map_err(|cause| Denial::new(ReceiverCause::Execution(cause), None))?;
        let opened = (|| {
            let context = context::prepare(&execution, self, binding, &mut plan, &allocation)?;
            if !plan.query.lanes().live_enabled()
                || context
                    .association
                    .access_contract()
                    .meaning()
                    .predicate_profile()
                    .is_err()
            {
                return Err(ReceiverCause::Refused(Refusal::UnsupportedMode));
            }
            let adapter = self
                .managed_indexes
                .get_admitted::<I>(&execution, &allocation)?
                .ok_or(ReceiverCause::Refused(Refusal::MissingProvider))?;
            if !adapter.installed().admits_association(&context.association) {
                return Err(ReceiverCause::Refused(Refusal::ForeignAssociation));
            }
            let graph = self
                .runtime
                .primary_graph()
                .ok_or(ReceiverCause::Refused(Refusal::ForeignGraph))?;
            let program = graph
                .layout
                .managed_source(I::NAME)
                .ok_or(ReceiverCause::Refused(Refusal::UnsupportedMode))?;
            let registry = graph.managed_derived_views();
            let observation = context.selected.product().observation().clone();
            let source = &self.primary_provider.live_delivery;
            let subscription = source
                .open_predicate(&execution, &observation)
                .map_err(live_denial)?;
            context.recheck_current(self)?;
            let audience = Audience::of(&plan.graph_work)?;
            let governance = plan.governance.authorization().map(|authorization| {
                crate::domain_computation::authorization::WorthQueryCapabilityUseIdentity::retain(authorization, &execution, &allocation)
            }).transpose()?;
            let maximum_results = plan.controls.maximum_result_count();
            let probe =
                RetainedPredicateProbe::capture(&execution, &context.association, &plan.parameters)
                    .map_err(work_denial)?;
            let reservation = registry
                .reserve_predicate_root(
                    &execution,
                    program,
                    PredicateWorld::of(&observation),
                    context.selector.subject(),
                    &context.association,
                )
                .map_err(work_denial)?;
            let mut read = self.read_managed_parent::<B>(
                &execution,
                plan,
                context,
                external,
                &allocation,
                ManagedParentCapture::Watch,
            )?;
            let selection = read
                .selection
                .take()
                .ok_or(ReceiverCause::Refused(Refusal::ForeignSelection))?;
            let opening = source
                .join_predicate_opening(&execution, &subscription, &observation)
                .map_err(live_denial)?;
            let slot = registry.issue_predicate_slot(&execution).map_err(|cause| {
                ReceiverCause::Provider(ProviderExecutionDenial::Lifecycle(cause))
            })?;
            let registration = reservation
                .install(&execution, slot, probe.clone())
                .map_err(work_denial)?;
            // Independent queue bookkeeping: this does not issue Parent READ.
            let basis = super::super::super::super::live::admit_live_managed_basis(
                self,
                1,
                &read.plan.graph_work,
                read.plan.query.name(),
                execution,
            )
            .map_err(ReceiverCause::LiveAdmission)?;
            let state = WatchState {
                registration,
                queue: WorthQueryLivePredicateQueue::new(subscription),
                basis: Some(basis),
                probe,
                selection,
                audience,
                maximum_results,
                governance,
            };
            let completed = self.finish_managed_parent::<B>(
                &execution,
                read,
                ManagedParentCompletionCapture::Watch {
                    state,
                    _opening: opening,
                },
            )?;
            let ManagedParentCompletionCapture::Watch { state, _opening } = completed.capture
            else {
                unreachable!("watch finalizer returns its exact capture");
            };
            drop(_opening);
            Ok((
                completed.read,
                WorthQueryManagedIndexWatch {
                    state: Some(state),
                    _binding: PhantomData,
                    _thread: PhantomData,
                },
            ))
        })();
        opened.map_err(|cause| Denial::new(cause, Some(allocation)))
    }
}
pub(super) fn work_denial(
    cause: super::super::super::super::WorthQueryManagedIndexWorkDenial,
) -> ReceiverCause {
    ReceiverCause::Provider(ProviderExecutionDenial::Work(cause))
}
pub(super) fn live_denial(cause: WorthQueryLiveAdmissionDenial) -> ReceiverCause {
    match cause {
        WorthQueryLiveAdmissionDenial::Execution(cause) => ReceiverCause::Execution(cause),
        WorthQueryLiveAdmissionDenial::Unavailable(_) => {
            ReceiverCause::Refused(Refusal::StaleImage)
        }
    }
}
