use super::PreparedPhysicalMutation;

impl PreparedPhysicalMutation {
    pub(in crate::physical_runtime) fn observe_existing_execution(
        &self,
    ) -> Option<crate::physical_runtime::durability::ExistingPhysicalMutationObservation> {
        self.start.observe_existing(self)
    }

    pub(in crate::physical_runtime) fn interruption(
        &self,
    ) -> Option<std::sync::Arc<worth_execution::ExecutionInterruptionView>> {
        self.start.interruption()
    }

    pub fn start(self) -> crate::physical_runtime::PhysicalMutationHandle {
        let start = self.start.clone();
        start.start(self)
    }

    pub(in crate::physical_runtime) fn start_anchored_step(
        self,
        step: std::sync::Arc<
            crate::physical_runtime::anchored_records::publication::step_custody::AcceptedStep,
        >,
    ) -> crate::physical_runtime::PhysicalMutationHandle {
        let start = self.start.clone();
        start.start_registered(
            self,
            Some(
                crate::physical_runtime::durability::ManagedPublicationRegistration::Anchored(step),
            ),
        )
    }

    pub(in crate::physical_runtime) fn start_ordered_journal(
        self,
        shared: std::sync::Arc<crate::physical_runtime::ordered_journal::JournalPublicationShared>,
    ) -> crate::physical_runtime::PhysicalMutationHandle {
        let start = self.start.clone();
        start.start_registered(
            self,
            Some(
                crate::physical_runtime::durability::ManagedPublicationRegistration::Journal(
                    shared,
                ),
            ),
        )
    }

    pub fn execute(self) -> crate::physical_runtime::PhysicalMutationOutcome {
        self.start().wait()
    }
}
