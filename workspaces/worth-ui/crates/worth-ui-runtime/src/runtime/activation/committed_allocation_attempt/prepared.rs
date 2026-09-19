pub(super) struct UiCommittedAllocationSuccessors {
    receipt_draft: super::WorthUiPlanSwapReceiptDraft,
    next_active: crate::runtime::active::WorthUiActiveRuntimeState,
    invalidation_transition:
        crate::runtime::invalidation_narrowing::UiPreparedInvalidationCatalogTransition,
    durable_resize_successor:
        crate::runtime::replacement::reconciliation::WorthUiDurableResizeSourceAuthority,
}

pub(crate) struct UiPreparedCommittedAllocationActivation {
    ledger_commit: crate::runtime::allocation_receipt::UiPreparedAllocationCatalogLedgerCommit,
    frame_commit: crate::runtime::allocation_frame_dispatch::UiPreparedFrameReplacementCommit,
    last_valid_successor: crate::runtime::launch::WorthUiLastValidRuntimeState,
    plan_swap: Box<crate::runtime::WorthUiPlanSwapReceipt>,
    next_active: crate::runtime::active::WorthUiActiveRuntimeState,
    invalidation_transition:
        crate::runtime::invalidation_narrowing::UiPreparedInvalidationCatalogTransition,
    frame_assignment: crate::runtime::allocation_frame_dispatch::UiAllocationFrameEpochAssignment,
    durable_resize_successor:
        crate::runtime::replacement::reconciliation::WorthUiDurableResizeSourceAuthority,
    query_succession: worth_ui_query_binding::WorthUiPreparedQueryBindingSuccession,
    successor_application_authority:
        crate::facade::prepared_application_authority::WorthUiPreparedApplicationLoweringAuthority,
    successor_planning_authority:
        std::rc::Rc<crate::runtime::WorthUiRetainedAllocationPlanningEvidenceRegistry>,
    application_publication:
        Option<crate::runtime::activation::WorthUiPreparedApplicationPublication>,
}

pub(crate) struct UiCommittedAllocationPublication {
    plan_swap: Box<crate::runtime::WorthUiPlanSwapReceipt>,
    query_retirement: worth_ui_query_binding::WorthUiOperationLiveRetirement,
    derived_index_counters: crate::runtime::invalidation_narrowing::UiDerivedIndexDeltaCounters,
}

pub(super) struct UiCommittedAllocationCommitResources {
    pub ledger_commit: crate::runtime::allocation_receipt::UiPreparedAllocationCatalogLedgerCommit,
    pub frame_commit: crate::runtime::allocation_frame_dispatch::UiPreparedFrameReplacementCommit,
    pub query_succession: worth_ui_query_binding::WorthUiPreparedQueryBindingSuccession,
    pub successor_application_authority:
        crate::facade::prepared_application_authority::WorthUiPreparedApplicationLoweringAuthority,
    pub successor_planning_authority:
        std::rc::Rc<crate::runtime::WorthUiRetainedAllocationPlanningEvidenceRegistry>,
    pub application_publication:
        Option<crate::runtime::activation::WorthUiPreparedApplicationPublication>,
    pub last_valid_successor: crate::runtime::launch::WorthUiLastValidRuntimeState,
}

impl UiCommittedAllocationSuccessors {
    pub(super) fn new(
        receipt_draft: super::WorthUiPlanSwapReceiptDraft,
        next_active: crate::runtime::active::WorthUiActiveRuntimeState,
        ledger_transition: crate::runtime::allocation_receipt::UiAllocationCatalogLedgerTransition,
        invalidation_transition: crate::runtime::invalidation_narrowing::UiPreparedInvalidationCatalogTransition,
    ) -> Self {
        let durable_resize_successor =
            crate::runtime::replacement::reconciliation::WorthUiDurableResizeSourceAuthority::prepare_successor(
                ledger_transition.durable_reconciliation(),
            );
        Self {
            receipt_draft,
            next_active,
            invalidation_transition,
            durable_resize_successor,
        }
    }

    pub(crate) fn bind_commit_resources(
        self,
        resources: UiCommittedAllocationCommitResources,
    ) -> UiPreparedCommittedAllocationActivation {
        let UiCommittedAllocationCommitResources {
            ledger_commit,
            mut frame_commit,
            query_succession,
            successor_application_authority,
            successor_planning_authority,
            application_publication,
            last_valid_successor,
        } = resources;
        let frame_assignment = frame_commit.assignment();
        let frame_transition = frame_commit.take_transition_for_receipt();
        let plan_swap = Box::new(self.receipt_draft.finish(frame_transition));
        UiPreparedCommittedAllocationActivation {
            ledger_commit,
            frame_commit,
            last_valid_successor,
            plan_swap,
            next_active: self.next_active,
            invalidation_transition: self.invalidation_transition,
            frame_assignment,
            durable_resize_successor: self.durable_resize_successor,
            query_succession,
            successor_application_authority,
            successor_planning_authority,
            application_publication,
        }
    }
}

impl UiPreparedCommittedAllocationActivation {
    pub(crate) fn candidate_replacement_authority(
        &self,
    ) -> Option<&crate::facade::prepared_application_authority::WorthUiPreparedApplicationAuthority>
    {
        self.application_publication
            .as_ref()?
            .replacement_authority()
    }

    pub(crate) fn candidate_plan(&self) -> &crate::runtime::WorthUiActiveExecutionPlan {
        self.next_active.active_plan_ref()
    }

    pub(crate) fn candidate_query_binding(
        &self,
    ) -> &worth_ui_query_binding::WorthUiRuntimeQueryBinding {
        self.query_succession.candidate()
    }

    pub(crate) fn candidate_allocation_catalog(
        &self,
    ) -> crate::runtime::UiMountedAllocationProjectionCatalog {
        self.ledger_commit.successor_mounted_projection_catalog()
    }

    pub(crate) fn candidate_plan_digest(&self) -> u64 {
        self.next_active.active_plan_ref().digest().as_u64()
    }

    pub(crate) fn candidate_allocation_truth_revision(&self) -> u64 {
        self.ledger_commit.successor_truth_revision().revision()
    }

    pub(crate) fn candidate_runtime_observation(
        &self,
    ) -> crate::runtime::WorthUiActiveRuntimeObservation {
        self.next_active.observation()
    }

    pub(crate) fn candidate_scheduler_state(
        &self,
    ) -> crate::runtime::UiAllocationFrameDispatcherState {
        self.frame_commit.successor_state()
    }

    pub(crate) fn previous_active_plan_digest(&self) -> u64 {
        self.plan_swap.previous_active_plan_digest()
    }

    pub(crate) fn commit_once(
        self,
        runtime: &mut crate::runtime::WorthUiRuntime,
        active_app: Option<&mut crate::facade::WorthUiApp>,
    ) -> UiCommittedAllocationPublication {
        let derived_index_counters = self.invalidation_transition.derived_index_counters();
        {
            let mut invalidation = runtime.allocation_invalidation_index.borrow_mut();
            assert_eq!(
                invalidation.active_catalog_identity_digest(),
                self.invalidation_transition.predecessor_identity_digest(),
                "reserved invalidation predecessor changed before total publication"
            );
            invalidation.commit_catalog_transition(self.invalidation_transition);
        }
        self.ledger_commit
            .commit_once(&runtime.allocation_receipt_ledger);
        advance_host_measurement_source(runtime, &self.plan_swap);
        runtime.last_valid = self.last_valid_successor;
        runtime.active = self.next_active;
        runtime
            .active
            .apply_allocation_frame_epoch_assignment(self.frame_assignment);
        self.frame_commit.commit_once();
        runtime.transient_interaction_admission = Default::default();
        runtime.durable_resize_source = self.durable_resize_successor;
        let query_retirement = self
            .query_succession
            .commit_once(&mut runtime.query_binding);
        runtime.active_application_lowering_authority = self.successor_application_authority;
        runtime.retained_allocation_planning_evidence = self.successor_planning_authority;
        if let Some(application_publication) = self.application_publication {
            application_publication.commit_once(
                active_app.expect("application replacement commit carries its active app"),
            );
        }
        UiCommittedAllocationPublication {
            plan_swap: self.plan_swap,
            query_retirement,
            derived_index_counters,
        }
    }
}

fn advance_host_measurement_source(
    runtime: &mut crate::runtime::WorthUiRuntime,
    plan_swap: &crate::runtime::WorthUiPlanSwapReceipt,
) {
    let mut source = runtime.host_measurement_source.borrow_mut();
    for receipt in plan_swap.committed_allocation().receipts() {
        for input in receipt
            .committed_allocation()
            .measurement_basis()
            .evidence_inputs()
        {
            if let Some(measurement) = input.as_host_measurement_result() {
                source.advance_past(measurement);
            }
        }
    }
}

impl UiCommittedAllocationPublication {
    pub(crate) fn into_parts(
        self,
    ) -> (
        Box<crate::runtime::WorthUiPlanSwapReceipt>,
        worth_ui_query_binding::WorthUiOperationLiveRetirement,
        crate::runtime::invalidation_narrowing::UiDerivedIndexDeltaCounters,
    ) {
        (
            self.plan_swap,
            self.query_retirement,
            self.derived_index_counters,
        )
    }
}
