use super::super::ports::RuntimeWorldObservationService;
use super::super::RuntimeWorldOwnerUnavailable;
use std::sync::{Arc, Weak};

/// Weak access to the owning World's observation service.
#[derive(Clone)]
pub struct RuntimeWorldObservationPort {
    owner: Weak<dyn RuntimeWorldObservationService + Send + Sync>,
}
impl RuntimeWorldObservationPort {
    /// Observe actual resident owner artifacts explicitly, without claiming
    /// Home durability or acquiring any performed/live result authority.
    pub fn observe_memory_original_publication(
        &self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        effect: worth_relational::facade::durability::ObservedRelationalMemoryCommit,
    ) -> Result<
        crate::history::WorldObservedMemoryPublication,
        super::super::RuntimeWorldServiceDenial<crate::history::WorldReadmittedPublicationDenial>,
    > {
        self.service()?
            .observe_memory_original_publication(execution, effect)
            .map_err(super::super::RuntimeWorldServiceDenial::Denied)
    }
    /// Read the original durable occurrence for this exact Native owner-admitted
    /// effect. Reconstruction reads the selected World image and bounded tails;
    /// it never executes a handler, publishes, or manufactures a live result.
    pub fn readmitted_native_publication(
        &self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        effect: worth_relational::facade::durability::ReadmittedRelationalAssociatedEffect,
    ) -> Result<
        crate::history::WorldReadmittedPublication,
        super::super::RuntimeWorldServiceDenial<crate::history::WorldReadmittedPublicationDenial>,
    > {
        self.service()?
            .readmitted_native_publication(execution, effect)
            .map_err(super::super::RuntimeWorldServiceDenial::Denied)
    }
    /// Lookup coordinates are durable data; an old process-issued identity is
    /// not a current-owner capability and cannot cross this boundary.
    ///
    /// ```compile_fail,E0308
    /// use worth_runtime_world::facade::{RuntimeWorldObservationPort, CompositeCommitIdentity};
    /// fn stale(port: &RuntimeWorldObservationPort, old: &CompositeCommitIdentity) {
    ///     let _ = port.readmitted_publication(old);
    /// }
    /// ```
    pub fn readmitted_publication(
        &self,
        identity: &worth_foundational::facade::DurableCompositeCommitIdentity,
    ) -> Result<
        crate::history::WorldReadmittedPublication,
        super::super::RuntimeWorldServiceDenial<crate::history::WorldReadmittedPublicationDenial>,
    > {
        self.service()?
            .readmitted_publication(identity)
            .map_err(super::super::RuntimeWorldServiceDenial::Denied)
    }
    pub fn observe_root_product_branch(
        &self,
    ) -> Result<
        crate::branch::ProductBranchObservation,
        super::super::RuntimeWorldServiceDenial<crate::branch::RuntimeWorldBranchAdmissionDenial>,
    > {
        self.service()?
            .observe_root_product_branch()
            .map_err(super::super::RuntimeWorldServiceDenial::Denied)
    }

    /// Resolve transported coordinates through this owner's live reference.
    /// The immutable selected commit supplies its exact component tuple.
    pub fn readmit_product_branch_reference(
        &self,
        description: &crate::branch::ProductBranchReferenceDescription,
    ) -> Result<
        crate::branch::ProductBranchObservation,
        super::super::RuntimeWorldServiceDenial<
            crate::branch::ProductBranchReferenceReadmissionDenial,
        >,
    > {
        self.service()?
            .readmit_product_branch_reference(description)
            .map_err(super::super::RuntimeWorldServiceDenial::Denied)
    }

    /// Install prepared state while both exact product heads remain current.
    /// All owner admission precedes the guarded section. The callback must not
    /// call either World owner or access either product cell; return custody
    /// for cleanup after the callback. Either refusal returns the argument.
    pub fn while_product_branches_current<Argument, Output>(
        &self,
        first: &crate::branch::ProductBranchObservation,
        second: &crate::branch::ProductBranchObservation,
        argument: Argument,
        install: impl for<'guard> FnOnce(
            Argument,
            crate::branch::CurrentProductHead<'guard>,
            crate::branch::CurrentProductHead<'guard>,
        ) -> Output,
    ) -> Result<Output, crate::branch::ProductBranchCurrentnessFailure<Argument>> {
        use crate::branch::ProductBranchCurrentnessFailure as Failure;
        let service = match self.service() {
            Ok(service) => service,
            Err(denial) => {
                return Err(Failure::AdmissionDenied {
                    denial: denial.into(),
                    argument,
                })
            }
        };
        let first = match service.admit_product_branch_currentness(first) {
            Ok(scope) => scope,
            Err(denial) => {
                return Err(Failure::AdmissionDenied {
                    denial: super::super::RuntimeWorldServiceDenial::Denied(denial),
                    argument,
                })
            }
        };
        let second = match service.admit_product_branch_currentness(second) {
            Ok(scope) => scope,
            Err(denial) => {
                return Err(Failure::AdmissionDenied {
                    denial: super::super::RuntimeWorldServiceDenial::Denied(denial),
                    argument,
                })
            }
        };
        first.while_both_current(second, argument, install)
    }

    /// Install prepared derived state while the exact issued head stays current.
    /// The callback receives currentness evidence but no mutation authority.
    /// Perform allocation/admission beforehand and return cleanup values from
    /// the callback; do not call this World owner inside the guarded section.
    pub fn while_product_branch_current<Argument, Output>(
        &self,
        expected: &crate::branch::ProductBranchObservation,
        argument: Argument,
        install: impl for<'guard> FnOnce(Argument, crate::branch::CurrentProductHead<'guard>) -> Output,
    ) -> Result<Output, crate::branch::ProductBranchCurrentnessFailure<Argument>> {
        let service = match self.service() {
            Ok(service) => service,
            Err(denial) => {
                return Err(
                    crate::branch::ProductBranchCurrentnessFailure::AdmissionDenied {
                        denial: denial.into(),
                        argument,
                    },
                )
            }
        };
        let scope = match service.admit_product_branch_currentness(expected) {
            Ok(scope) => scope,
            Err(denial) => {
                return Err(
                    crate::branch::ProductBranchCurrentnessFailure::AdmissionDenied {
                        denial: super::super::RuntimeWorldServiceDenial::Denied(denial),
                        argument,
                    },
                )
            }
        };
        scope.while_current(argument, install)
    }

    pub(in crate::lifecycle) fn new(
        owner: Weak<dyn RuntimeWorldObservationService + Send + Sync>,
    ) -> Self {
        Self { owner }
    }
    fn service(
        &self,
    ) -> Result<Arc<dyn RuntimeWorldObservationService + Send + Sync>, RuntimeWorldOwnerUnavailable>
    {
        let service = self
            .owner
            .upgrade()
            .ok_or_else(RuntimeWorldOwnerUnavailable::new)?;
        if !service.is_available() {
            return Err(RuntimeWorldOwnerUnavailable::new());
        }
        Ok(service)
    }
    pub fn observe_product_branch(
        &self,
        branch: &crate::identity::ProductBranchIdentity,
    ) -> Result<
        crate::branch::ProductBranchObservation,
        super::super::RuntimeWorldServiceDenial<crate::branch::RuntimeWorldBranchAdmissionDenial>,
    > {
        self.service()?
            .observe_product_branch(branch)
            .map_err(super::super::RuntimeWorldServiceDenial::Denied)
    }

    /// Resolves one owner-issued live occurrence without exposing or
    /// reconstructing its name-keyed product identity.
    pub fn observe_product_branch_occurrence(
        &self,
        occurrence: crate::identity::ProductBranchIncarnation,
    ) -> Result<
        crate::branch::ProductBranchObservation,
        super::super::RuntimeWorldServiceDenial<crate::branch::RuntimeWorldBranchAdmissionDenial>,
    > {
        self.service()?
            .observe_product_branch_occurrence(occurrence)
            .map_err(super::super::RuntimeWorldServiceDenial::Denied)
    }

    /// Selects one retained source on this exact target's inherited ancestry.
    /// Every intervening commit must remain retained; a raw-parent gap refuses.
    /// Traversal stays under the original execution request.
    pub fn select_product_branch_ancestor(
        &self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        target: &crate::branch::ProductBranchObservation,
        source: &crate::identity::CompositeCommitIdentity,
        maximum: Option<std::num::NonZeroUsize>,
    ) -> Result<
        crate::branch::ProductBranchAncestorSelection,
        super::super::RuntimeWorldServiceDenial<
            crate::branch::ProductBranchAncestorSelectionDenial,
        >,
    > {
        self.service()?
            .select_product_branch_ancestor(execution, target, source, maximum)
            .map_err(super::super::RuntimeWorldServiceDenial::Denied)
    }

    /// Proves ancestry between retained endpoints using owner-recorded retirement
    /// splices. This selects no retired contents and grants no exact-history page.
    /// Work is bounded by retained edges under the original execution request.
    pub fn select_retained_product_branch_ancestor(
        &self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        target: &crate::branch::ProductBranchObservation,
        source: &crate::identity::CompositeCommitIdentity,
        maximum: Option<std::num::NonZeroUsize>,
    ) -> Result<
        crate::branch::ProductBranchAncestorSelection,
        super::super::RuntimeWorldServiceDenial<
            crate::branch::ProductBranchAncestorSelectionDenial,
        >,
    > {
        self.service()?
            .select_retained_product_branch_ancestor(execution, target, source, maximum)
            .map_err(super::super::RuntimeWorldServiceDenial::Denied)
    }

    /// Open at exactly the retained observation, with one catalog admission
    /// probe. No live branch head or reference generation is substituted.
    pub fn open_exact_product_history(
        &self,
        execution: worth_execution::ExecutionRequest<'_, '_>,
        target: &crate::branch::ProductBranchObservation,
    ) -> Result<
        crate::branch::ProductExactHistoryCursor,
        super::super::RuntimeWorldServiceDenial<crate::branch::ProductExactHistoryDenial>,
    > {
        self.service()?
            .open_exact_product_history(execution, target)
            .map_err(super::super::RuntimeWorldServiceDenial::Denied)
    }

    /// Prepare one genuine product-parent step. Its sealed result retains the
    /// originating request and can be declined without consuming the cursor.
    pub fn prepare_exact_product_history_page<'request, 'authority>(
        &self,
        execution: worth_execution::ExecutionRequest<'request, 'authority>,
        cursor: &crate::branch::ProductExactHistoryCursor,
    ) -> Result<
        crate::branch::PreparedProductExactHistoryPage<'request, 'authority>,
        super::super::RuntimeWorldServiceDenial<crate::branch::ProductExactHistoryDenial>,
    > {
        self.service()?
            .prepare_exact_product_history_page(execution, cursor)
            .map_err(super::super::RuntimeWorldServiceDenial::Denied)
    }

    pub fn trace_product_branch_ancestry(
        &self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        occurrence: crate::identity::ProductBranchIncarnation,
        maximum: Option<std::num::NonZeroUsize>,
    ) -> Result<
        crate::branch::ProductBranchHistoryTraversal,
        super::super::RuntimeWorldServiceDenial<crate::branch::RuntimeWorldBranchAdmissionDenial>,
    > {
        self.service()?
            .trace_product_branch_ancestry(execution, occurrence, maximum)
            .map_err(super::super::RuntimeWorldServiceDenial::Denied)
    }

    pub fn continue_product_branch_ancestry(
        &self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        previous: &crate::branch::ProductBranchHistoryTraversal,
        maximum: Option<std::num::NonZeroUsize>,
    ) -> Result<
        crate::branch::ProductBranchHistoryTraversal,
        super::super::RuntimeWorldServiceDenial<crate::branch::RuntimeWorldBranchAdmissionDenial>,
    > {
        self.service()?
            .continue_product_branch_ancestry(execution, previous, maximum)
            .map_err(super::super::RuntimeWorldServiceDenial::Denied)
    }

    pub fn observe_product_branch_history_entry(
        &self,
        history: &crate::branch::ProductBranchHistoryTraversal,
        index: usize,
    ) -> Result<
        crate::branch::ProductBranchObservation,
        super::super::RuntimeWorldServiceDenial<crate::branch::RuntimeWorldBranchAdmissionDenial>,
    > {
        self.service()?
            .observe_product_branch_history_entry(history, index)
            .map_err(super::super::RuntimeWorldServiceDenial::Denied)
    }
}
