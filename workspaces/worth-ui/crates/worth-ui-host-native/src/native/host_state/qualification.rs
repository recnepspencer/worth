use std::collections::BTreeSet;

#[derive(Clone, Copy)]
pub(crate) struct UiNativePresentationExternalQualification {
    effects_indeterminate: bool,
    duplicate_completed: bool,
}

impl UiNativePresentationExternalQualification {
    pub(crate) const fn effects_indeterminate(self) -> bool {
        self.effects_indeterminate
    }

    pub(crate) const fn duplicate_completed(self) -> bool {
        self.duplicate_completed
    }
}

/// A planned qualification fault fires once, at the ordinal the plan names.
#[derive(Clone, Copy)]
enum UiNativePlannedInjection {
    Unplanned,
    Armed { ordinal: u64 },
    Spent,
}

impl UiNativePlannedInjection {
    const fn from_plan(ordinal: Option<u64>) -> Self {
        match ordinal {
            Some(ordinal) => Self::Armed { ordinal },
            None => Self::Unplanned,
        }
    }

    const fn fires_at(self, current: u64) -> bool {
        matches!(self, Self::Armed { ordinal } if ordinal == current)
    }
}

/// A derived-state loss whose predecessor bindings have not all been restored.
struct UiNativePendingReconstruction {
    class: crate::UiNativeDerivedStateLossClass,
    predecessors: BTreeSet<u64>,
    successors: BTreeSet<u64>,
}

pub(crate) struct UiNativeQualificationState {
    deferred_presentations: [Option<u64>; 3],
    duplicate_completion: UiNativePlannedInjection,
    effects_indeterminate: UiNativePlannedInjection,
    presentation_submission_count: u64,
    derived_state_loss: Option<crate::UiNativeDerivedStateLossClass>,
    completed_derived_state_loss: UiNativePlannedInjection,
    derived_state_loss_pending: Option<crate::UiNativeDerivedStateLossClass>,
    derived_state_reconstruction: Option<UiNativePendingReconstruction>,
    derived_state_loss_count: u64,
    derived_state_reconstruction_count: u64,
    surface_basis_successor:
        Option<crate::qualification::UiNativeQualificationSurfaceBasisSuccessor>,
}

impl UiNativeQualificationState {
    pub(super) const fn ordinary() -> Self {
        Self {
            deferred_presentations: [None, None, None],
            duplicate_completion: UiNativePlannedInjection::Unplanned,
            effects_indeterminate: UiNativePlannedInjection::Unplanned,
            presentation_submission_count: 0,
            derived_state_loss: None,
            completed_derived_state_loss: UiNativePlannedInjection::Unplanned,
            derived_state_loss_pending: None,
            derived_state_reconstruction: None,
            derived_state_loss_count: 0,
            derived_state_reconstruction_count: 0,
            surface_basis_successor: None,
        }
    }

    pub(super) fn from_plan(plan: crate::UiNativeQualificationPlan) -> Self {
        Self {
            deferred_presentations: plan.deferred_presentations(),
            duplicate_completion: UiNativePlannedInjection::from_plan(
                plan.duplicate_completion_presentation(),
            ),
            effects_indeterminate: UiNativePlannedInjection::from_plan(
                plan.effects_indeterminate_presentation(),
            ),
            presentation_submission_count: 0,
            derived_state_loss: plan.derived_state_loss(),
            completed_derived_state_loss: UiNativePlannedInjection::from_plan(
                plan.completed_derived_state_loss_ordinal(),
            ),
            derived_state_loss_pending: None,
            derived_state_reconstruction: None,
            derived_state_loss_count: 0,
            derived_state_reconstruction_count: 0,
            surface_basis_successor: plan.surface_basis_successor(),
        }
    }

    pub(crate) fn defer_next_presentation_initial_observation(&mut self) -> bool {
        self.presentation_submission_count = self.presentation_submission_count.saturating_add(1);
        self.deferred_presentations
            .contains(&Some(self.presentation_submission_count))
    }

    pub(crate) fn presentation_external_qualification(
        &self,
        identity: super::super::physical_work_signal::UiNativePhysicalPresentationIdentity,
    ) -> UiNativePresentationExternalQualification {
        UiNativePresentationExternalQualification {
            effects_indeterminate: self.effects_indeterminate.fires_at(identity.sequence()),
            duplicate_completed: self.duplicate_completion.fires_at(identity.sequence()),
        }
    }

    pub(crate) fn presentation_poll_override(
        &self,
        identity: super::super::physical_work_signal::UiNativePhysicalPresentationIdentity,
    ) -> Option<(
        super::super::physical_work_signal::UiNativePhysicalSignalStatus,
        Option<crate::UiNativeDerivedStateLossClass>,
    )> {
        self.effects_indeterminate
            .fires_at(identity.sequence())
            .then_some((
            super::super::physical_work_signal::UiNativePhysicalSignalStatus::EffectsIndeterminate,
            self.derived_state_loss,
        ))
    }

    pub(crate) fn should_duplicate_completed_observation(
        &self,
        identity: super::super::physical_work_signal::UiNativePhysicalPresentationIdentity,
    ) -> bool {
        self.duplicate_completion.fires_at(identity.sequence())
    }

    pub(crate) fn commit_duplicate_completed_observation(
        &mut self,
        identity: super::super::physical_work_signal::UiNativePhysicalPresentationIdentity,
    ) {
        assert!(self.should_duplicate_completed_observation(identity));
        self.duplicate_completion = UiNativePlannedInjection::Spent;
    }

    pub(crate) fn commit_presentation_poll_override(
        &mut self,
        identity: super::super::physical_work_signal::UiNativePhysicalPresentationIdentity,
    ) {
        assert!(
            self.presentation_poll_override(identity).is_some(),
            "only the selected owner observation may commit the qualification override"
        );
        self.effects_indeterminate = UiNativePlannedInjection::Spent;
        self.derived_state_loss_pending = self.derived_state_loss;
    }

    pub(crate) fn take_derived_state_loss(
        &mut self,
    ) -> Option<crate::UiNativeDerivedStateLossClass> {
        self.derived_state_loss_pending.take()
    }

    pub(crate) fn completed_derived_state_loss(
        &self,
        completed_presentations: u64,
    ) -> Option<crate::UiNativeDerivedStateLossClass> {
        self.completed_derived_state_loss
            .fires_at(completed_presentations)
            .then_some(self.derived_state_loss)
            .flatten()
    }

    pub(crate) fn commit_completed_derived_state_loss(&mut self) {
        assert!(matches!(
            self.completed_derived_state_loss,
            UiNativePlannedInjection::Armed { .. }
        ));
        self.completed_derived_state_loss = UiNativePlannedInjection::Spent;
    }

    pub(crate) fn record_derived_state_loss(
        &mut self,
        class: crate::UiNativeDerivedStateLossClass,
        bindings: BTreeSet<u64>,
    ) {
        assert_eq!(self.derived_state_loss, Some(class));
        assert!(!bindings.is_empty());
        self.derived_state_reconstruction = Some(UiNativePendingReconstruction {
            class,
            predecessors: bindings,
            successors: BTreeSet::new(),
        });
        self.derived_state_loss_count = self.derived_state_loss_count.saturating_add(1);
    }

    pub(crate) fn pending_reconstruction(&self) -> Option<crate::UiNativeDerivedStateLossClass> {
        self.derived_state_reconstruction
            .as_ref()
            .map(|pending| pending.class)
    }

    pub(crate) fn record_derived_state_reconstruction(&mut self, binding: u64, restored: bool) {
        let Some(pending) = self.derived_state_reconstruction.as_mut() else {
            return;
        };
        if !restored || !pending.successors.insert(binding) {
            return;
        }
        let _ = pending.predecessors.pop_first();
        if pending.predecessors.is_empty() {
            self.derived_state_reconstruction = None;
            self.derived_state_reconstruction_count =
                self.derived_state_reconstruction_count.saturating_add(1);
        }
    }

    pub(crate) const fn derived_state_reconstruction_observation(
        &self,
    ) -> Option<crate::UiNativeDerivedStateReconstructionObservation> {
        let Some(class) = self.derived_state_loss else {
            return None;
        };
        Some(
            crate::UiNativeDerivedStateReconstructionObservation::observed(
                class,
                self.derived_state_loss_count,
                self.derived_state_reconstruction_count,
            ),
        )
    }

    pub(crate) fn take_surface_basis_successor(
        &mut self,
        completed_presentations: u64,
    ) -> Option<crate::qualification::UiNativeQualificationSurfaceBasisSuccessor> {
        let successor = self.surface_basis_successor?;
        if completed_presentations < successor.after_completed_presentation() {
            return None;
        }
        self.surface_basis_successor.take()
    }
}
