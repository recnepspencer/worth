use crate::runtime::launch::runtime_instance::WorthUiRuntime;
use crate::runtime::UiAllocationFrameGatewayOutcome;

use super::WorthUiFrameworkTurnCompletion;
/// The single ordinary production turn that may collect allocation sources.
///
/// Source callbacks receive this borrowed capability, never the runtime or the
/// dispatcher. When the callback returns, `WorthUiRuntime::execute_framework_turn`
/// closes and pumps exactly once before yielding an activation boundary.
pub struct WorthUiFrameworkTurn<'runtime> {
    pub(super) runtime: &'runtime mut WorthUiRuntime,
}

pub struct WorthUiHostMeasurementTurnSource<'turn> {
    pub(super) runtime: &'turn mut WorthUiRuntime,
}

pub struct WorthUiQueryProjectionTurnSource<'turn> {
    pub(super) runtime: &'turn mut WorthUiRuntime,
}

pub struct WorthUiInteractionTurnSource<'turn> {
    pub(super) runtime: &'turn mut WorthUiRuntime,
}

pub struct WorthUiResizePreviewTurnSource<'turn> {
    pub(super) runtime: &'turn mut WorthUiRuntime,
}

pub struct WorthUiScrollExtentTurnSource<'turn> {
    pub(super) runtime: &'turn mut WorthUiRuntime,
}

pub struct WorthUiDurableResizeTurnSource<'turn> {
    pub(super) runtime: &'turn mut WorthUiRuntime,
}

/// ```compile_fail
/// use worth_ui_runtime::facade::runtime_handoff::WorthUiRuntime;
///
/// fn caller_cannot_pump_runtime(runtime: &mut WorthUiRuntime) {
///     runtime.run_allocation_frame_turn();
/// }
/// ```
/// Production owner of source capabilities and framework turns. Source
/// capabilities enqueue through the dispatcher and cannot close it.
/// The framework boundary advances only through `WorthUiRuntimeFrameworkLoop`.
impl WorthUiRuntime {
    /// Run one ordinary framework turn and close/pump it exactly once.
    ///
    /// The borrowed turn exposes source-family admission only. It cannot close,
    /// pump, classify policy, acknowledge a handoff, or commit a receipt.
    ///
    /// ```compile_fail
    /// use worth_ui_runtime::facade::runtime_handoff::WorthUiRuntime;
    ///
    /// fn caller_cannot_clock_allocation_turn(runtime: &mut WorthUiRuntime) {
    ///     runtime.framework_turn(|_| {});
    /// }
    /// ```
    pub fn execute_framework_turn(
        &mut self,
        collect_sources: impl FnOnce(&mut WorthUiFrameworkTurn<'_>),
    ) -> WorthUiFrameworkTurnCompletion<'_> {
        let mut collect_sources = Some(collect_sources);
        self.finish_framework_turn(&mut |runtime| {
            collect_sources.take().expect("source collection runs once")(
                &mut WorthUiFrameworkTurn { runtime },
            );
        })
    }

    /// Source collection is the only typed callback. Panic containment and the
    /// close/pump that follows it are compiled once.
    #[inline(never)]
    fn finish_framework_turn(
        &mut self,
        collect_sources: &mut dyn FnMut(&mut WorthUiRuntime),
    ) -> WorthUiFrameworkTurnCompletion<'_> {
        let collection = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            collect_sources(self);
        }));
        let turn = self.close_allocation_ingress_and_pump_once();
        let transition = self.plan_framework_transition(turn);
        match collection {
            Ok(()) => self.execute_transition_or_publish_denial(transition),
            Err(payload) => {
                self.acknowledge_discarded_transition(transition);
                std::panic::resume_unwind(payload);
            }
        }
    }

    fn close_allocation_ingress_and_pump_once(&self) -> super::UiAllocationFrameTurnOutcome {
        self.allocation_frame_scheduler.run_turn()
    }

    fn plan_framework_transition(
        &self,
        turn: super::UiAllocationFrameTurnOutcome,
    ) -> super::transition_planning::UiFrameworkTransitionPlanningDisposition {
        super::transition_planning::plan_framework_transition(
            turn,
            self.active.generation_identity(),
            self.active.frame_epoch(),
            &self.allocation_source_order_ledger,
            &self.allocation_receipt_ledger,
            &self.allocation_invalidation_index,
        )
    }

    fn execute_transition_or_publish_denial(
        &mut self,
        transition: super::transition_planning::UiFrameworkTransitionPlanningDisposition,
    ) -> WorthUiFrameworkTurnCompletion<'_> {
        match transition {
            super::transition_planning::UiFrameworkTransitionPlanningDisposition::NoIngress {
                active_generation,
                active_frame_epoch,
            } => super::execution::execute_no_ingress_framework_transition(
                self,
                active_generation,
                active_frame_epoch,
            ),
            super::transition_planning::UiFrameworkTransitionPlanningDisposition::Planned(plan) => {
                super::execution::execute_planned_framework_transition(self, *plan)
            }
            super::transition_planning::UiFrameworkTransitionPlanningDisposition::FrameResolutionDenied(rejection) => {
                WorthUiFrameworkTurnCompletion::AllocationFrameResolutionDenied { rejection }
            }
            super::transition_planning::UiFrameworkTransitionPlanningDisposition::InvalidationNarrowingDenied(rejection) => {
                WorthUiFrameworkTurnCompletion::AllocationInvalidationNarrowingDenied { rejection }
            }
            super::transition_planning::UiFrameworkTransitionPlanningDisposition::ReplanSelectionDenied(denial) => {
                WorthUiFrameworkTurnCompletion::AllocationReplanSelectionDenied { denial }
            }
            super::transition_planning::UiFrameworkTransitionPlanningDisposition::PlanningDenied(denial) => {
                WorthUiFrameworkTurnCompletion::FrameworkTransitionPlanningDenied { denial }
            }
            super::transition_planning::UiFrameworkTransitionPlanningDisposition::DispatchDenied { denial, counters } => {
                WorthUiFrameworkTurnCompletion::Denied { denial, counters }
            }
        }
    }

    fn acknowledge_discarded_transition(
        &mut self,
        transition: super::transition_planning::UiFrameworkTransitionPlanningDisposition,
    ) {
        match transition {
            super::transition_planning::UiFrameworkTransitionPlanningDisposition::NoIngress {
                ..
            } => {}
            super::transition_planning::UiFrameworkTransitionPlanningDisposition::Planned(
                planned,
            ) => super::execution::acknowledge_discarded_framework_transition(self, *planned),
            super::transition_planning::UiFrameworkTransitionPlanningDisposition::FrameResolutionDenied(_)
            | super::transition_planning::UiFrameworkTransitionPlanningDisposition::InvalidationNarrowingDenied(_)
            | super::transition_planning::UiFrameworkTransitionPlanningDisposition::ReplanSelectionDenied(_)
            | super::transition_planning::UiFrameworkTransitionPlanningDisposition::PlanningDenied(_)
            | super::transition_planning::UiFrameworkTransitionPlanningDisposition::DispatchDenied {
                ..
            } => {}
        }
    }

    pub(crate) fn host_measurement_collector(
        &self,
    ) -> crate::host::WorthUiHostMeasurementCollector {
        crate::host::WorthUiHostMeasurementCollector::new(std::rc::Rc::clone(
            &self.host_measurement_source,
        ))
    }

    pub(super) fn collect_and_submit_host_measurement<
        A: worth_ui_host_contract::WorthUiMeasurementHostAdapter + ?Sized,
    >(
        &self,
        adapter: &A,
        input: crate::host::UiHostMeasurementCollectionInput<'_>,
    ) -> Result<UiAllocationFrameGatewayOutcome, crate::host::UiHostMeasurementEvidenceDenial> {
        let admitted = self
            .host_measurement_collector()
            .collect_admitted(adapter, input)?;
        Ok(self
            .host_measurement_submission()
            .submit_admitted_host_measurement(admitted))
    }

    pub(super) fn admit_settled_query_projection(
        &mut self,
        projection: worth_ui_query_binding::WorthUiSettledSnapshotProjection,
    ) -> Result<
        std::sync::Arc<worth_ui_query_binding::WorthUiSettledSnapshotFact>,
        worth_ui_query_binding::WorthUiSettledSnapshotAdmissionStop,
    > {
        self.query_binding.admit_settled_snapshot(projection)
    }

    pub(super) fn refresh_settled_query_projection(
        &mut self,
        projection: worth_ui_query_binding::WorthUiSettledSnapshotProjection,
    ) -> Result<
        std::sync::Arc<worth_ui_query_binding::WorthUiSettledSnapshotFact>,
        worth_ui_query_binding::WorthUiSettledSnapshotAdmissionStop,
    > {
        self.query_binding.refresh_settled_snapshot(projection)
    }

    pub(super) fn submit_settled_query_fact(
        &self,
        link: &crate::runtime::WorthUiQueryLaneFactLink,
    ) -> Result<
        crate::runtime::WorthUiQueryFrameIngressOutcome,
        crate::runtime::WorthUiQueryFrameIngressDenial,
    > {
        let mut counters = crate::runtime::WorthUiQueryFrameIngressCounters::default();
        counters.record_link_resolution();
        if !link.belongs_to_generation(
            &self
                .active_application_lowering_authority
                .generation_witness(),
        ) {
            return Err(crate::runtime::WorthUiQueryFrameIngressDenial::StaleApplicationGeneration);
        }
        let active_link = self
            .active
            .active_plan_ref()
            .query_fact_link_for_plan_index(link.plan_index())
            .ok_or(crate::runtime::WorthUiQueryFrameIngressDenial::PlanRowNotActive)?;
        if active_link != *link.settled_fact_link() {
            return Err(crate::runtime::WorthUiQueryFrameIngressDenial::PlanBindingMismatch);
        }
        let fact = self
            .query_binding
            .settled_snapshot_fact_reference_for(active_link.installed_reference())
            .map_err(crate::runtime::WorthUiQueryFrameIngressDenial::RetainedFact)?;
        counters.record_retained_fact_resolution();
        let gateway = self.query_settled_fact_submission().submit(
            link.plan_index(),
            crate::capability::ViewBindingId::new(link.view_binding_id())
                .expect("active plan binding identities remain valid"),
            fact,
        );
        counters.record_allocation_submission();
        Ok(crate::runtime::WorthUiQueryFrameIngressOutcome::new(
            gateway, counters,
        ))
    }

    pub(super) fn admit_operation_live(
        &mut self,
        resource: worth_ui_query_binding::WorthUiOperationLiveResource,
    ) -> Result<(), worth_ui_query_binding::WorthUiOperationLiveAdmissionStop> {
        self.query_binding.admit_operation_live(resource)
    }

    pub(super) fn admit_operation_live_change(
        &mut self,
        consequence: worth_ui_query_binding::WorthUiCollectionChangeConsequence,
    ) -> Result<
        worth_ui_query_binding::WorthUiCollectionChangeStagingReceipt,
        worth_ui_query_binding::WorthUiCollectionChangeAdmissionStop,
    > {
        self.query_binding.admit_operation_live_change(consequence)
    }

    pub(crate) fn validate_operation_live_change_observation(
        &self,
        consequence: worth_ui_query_binding::WorthUiCollectionChangeConsequence,
    ) -> Result<
        worth_ui_query_binding::WorthUiValidatedCollectionChangeObservation,
        worth_ui_query_binding::WorthUiCollectionChangeAdmissionStop,
    > {
        self.query_binding
            .validate_operation_live_change_observation(consequence)
    }

    pub(super) fn refresh_and_admit_operation_live(
        &mut self,
        request: worth_ui_query_binding::WorthUiOperationLiveRefreshRequest<'_>,
    ) -> Result<
        worth_ui_query_binding::WorthUiOperationLiveSourceRefreshOutcome,
        worth_ui_query_binding::WorthUiOperationLiveSourceRefreshStop,
    > {
        match self.query_binding.refresh_operation_live(request) {
            Ok(worth_ui_query_binding::WorthUiOperationLiveRefreshOutcome::NoSemanticDelivery) => {
                Ok(
                    worth_ui_query_binding::WorthUiOperationLiveSourceRefreshOutcome::NoSemanticDelivery,
                )
            }
            Ok(worth_ui_query_binding::WorthUiOperationLiveRefreshOutcome::Applied(
                consequence,
            )) => self
                .admit_operation_live_change(consequence)
                .map(worth_ui_query_binding::WorthUiOperationLiveSourceRefreshOutcome::Staged)
                .map_err(|stop| {
                    worth_ui_query_binding::WorthUiOperationLiveSourceRefreshStop::Publication(
                        Box::new(stop),
                    )
                }),
            Err(stop) => Err(
                worth_ui_query_binding::WorthUiOperationLiveSourceRefreshStop::Progression(
                    Box::new(stop),
                ),
            ),
        }
    }

    pub(super) fn admit_and_submit_interaction(
        &mut self,
        target: crate::graph::UiGraphNodeIdentity,
        state: crate::runtime::WorthUiTransientInteractionState,
    ) -> Result<
        UiAllocationFrameGatewayOutcome,
        crate::runtime::WorthUiTransientInteractionAdmissionDenial,
    > {
        let admitted = self.transient_interaction_admission.admit(target, state)?;
        Ok(self
            .interaction_submission()
            .submit_admitted_transient_interaction(admitted))
    }

    pub(super) fn admit_and_submit_resize_preview(
        &mut self,
        sample: crate::runtime::UiResizePreviewSample,
    ) -> Result<
        UiAllocationFrameGatewayOutcome,
        crate::runtime::WorthUiTransientInteractionAdmissionDenial,
    > {
        let admitted = self
            .transient_interaction_admission
            .admit_resize_preview(sample)?;
        Ok(self
            .interaction_submission()
            .submit_admitted_transient_interaction(admitted))
    }

    pub(super) fn admit_and_submit_durable_resize(
        &mut self,
        input: crate::runtime::UiDurableResizeCommitIntent,
    ) -> Result<
        UiAllocationFrameGatewayOutcome,
        crate::runtime::WorthUiDurableResizeSourceAdmissionDenial,
    > {
        let admitted = self.durable_resize_source.admit(input)?;
        Ok(self
            .durable_resize_submission()
            .submit_admitted_durable_resize(admitted))
    }

    pub(in crate::runtime::allocation_frame_dispatch) fn host_measurement_submission(
        &self,
    ) -> super::super::gateway::WorthUiHostMeasurementSubmission {
        super::super::gateway::WorthUiHostMeasurementSubmission::new(
            self.allocation_frame_scheduler.mailbox(),
        )
    }

    pub(in crate::runtime::allocation_frame_dispatch) fn query_settled_fact_submission(
        &self,
    ) -> super::super::gateway::WorthUiQuerySettledFactSubmission {
        super::super::gateway::WorthUiQuerySettledFactSubmission::new(
            self.allocation_frame_scheduler.mailbox(),
        )
    }

    pub(in crate::runtime::allocation_frame_dispatch) fn interaction_submission(
        &self,
    ) -> super::super::gateway::WorthUiInteractionSubmission {
        super::super::gateway::WorthUiInteractionSubmission::new(
            self.allocation_frame_scheduler.mailbox(),
        )
    }

    pub(in crate::runtime::allocation_frame_dispatch) fn durable_resize_submission(
        &self,
    ) -> super::super::gateway::WorthUiDurableResizeSubmission {
        super::super::gateway::WorthUiDurableResizeSubmission::new(
            self.allocation_frame_scheduler.mailbox(),
        )
    }
}
