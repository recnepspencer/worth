use super::{UiMountedFramePreparationDenial as Denial, UiPreparedMountedFrame};
use crate::mounting::projection::UiMountedPointerAffordanceWork;

impl UiPreparedMountedFrame {
    pub(crate) fn stage_pointer_affordance(
        &mut self,
        snapshot: Option<&crate::runtime::pointer_affordance::UiPointerAffordanceSnapshot>,
        mounted: &crate::mounting::WorthUiMountedSessionState,
    ) -> Result<(), Denial> {
        self.stage_pointer_affordance_snapshot(snapshot, mounted, true)
    }

    #[cfg(test)]
    pub(crate) fn stage_untrusted_pointer_affordance_for_test(
        &mut self,
        snapshot: Option<&crate::runtime::pointer_affordance::UiPointerAffordanceSnapshot>,
        mounted: &crate::mounting::WorthUiMountedSessionState,
    ) -> Result<(), Denial> {
        self.stage_pointer_affordance_snapshot(snapshot, mounted, false)
    }

    fn stage_pointer_affordance_snapshot(
        &mut self,
        snapshot: Option<&crate::runtime::pointer_affordance::UiPointerAffordanceSnapshot>,
        mounted: &crate::mounting::WorthUiMountedSessionState,
        snapshot_is_current: bool,
    ) -> Result<(), Denial> {
        let mut desired = Vec::new();
        let mut admitted_surfaces = Vec::new();
        let mut work = UiMountedPointerAffordanceWork::default();
        if let Some(snapshot) = snapshot {
            if snapshot.generation().prepared_generation() != &self.generation {
                return Err(Denial::PointerSnapshotGenerationMismatch);
            }
            for projection in snapshot.active_projections() {
                work.observations_examined += 1;
                let current_observation =
                    mounted.pointer_projection_is_current(snapshot, projection);
                if current_observation {
                    admitted_surfaces.push(projection.surface());
                }
                if !self
                    .manifest
                    .surfaces()
                    .iter()
                    .any(|binding| binding.semantic_surface() == projection.surface())
                {
                    continue;
                }
                let Some(target) = projection.presented_target() else {
                    continue;
                };
                work.targets_examined += 1;
                let instance = target.mounted_instance();
                let (candidate, probes) = self
                    .candidate
                    .frame()
                    .semantic_projection()
                    .node_receipt_with_probes(instance);
                work.membership_key_probes += probes;
                // Candidate membership owns departure, even without new input.
                let Some(candidate) = candidate else { continue };
                let denial = || Denial::PointerSnapshotTargetUnavailable(instance);
                if candidate.semantic_surface() != projection.surface() {
                    return Err(denial());
                }
                let Some(current) = mounted.current_mounted_identity_basis(instance) else {
                    continue;
                };
                if candidate.incarnation() != current.mount_incarnation() {
                    continue;
                }
                if !current_observation && !snapshot_is_current {
                    return Err(denial());
                }
                if snapshot_is_current && !current_observation {
                    admitted_surfaces.push(projection.surface());
                }
                let family = match projection.family() {
                    crate::declaration::UiPointerAffordance::Default => {
                        worth_ui_host_contract::UiPointerAffordanceFamily::Default
                    }
                    crate::declaration::UiPointerAffordance::Activation => {
                        worth_ui_host_contract::UiPointerAffordanceFamily::Activation
                    }
                };
                desired.push(worth_ui_host_contract::UiMountedPointerAffordanceMechanic::complete_from_runtime_mounting(
                    projection.pointer(), projection.surface(), instance, family,
                ));
            }
        }
        self.candidate.owner.pointer.stage(
            desired,
            work,
            self.manifest
                .surfaces()
                .iter()
                .map(|binding| binding.semantic_surface()),
            snapshot,
            admitted_surfaces,
        );
        Ok(())
    }

    pub(crate) fn clear_pointer_affordance(&mut self) {
        self.candidate.owner.pointer.stage(
            Vec::new(),
            UiMountedPointerAffordanceWork::default(),
            self.manifest
                .surfaces()
                .iter()
                .map(|binding| binding.semantic_surface()),
            None,
            Vec::new(),
        );
    }

    #[cfg(test)]
    pub(crate) fn pointer_affordance_work(&self) -> UiMountedPointerAffordanceWork {
        self.candidate.owner.pointer.work()
    }

    #[cfg(test)]
    pub(crate) fn verify_pointer_output_retry(&self) {
        let mut rejected = self.candidate.owner.clone_for_appearance_output_test();
        let mut control = self.candidate.owner.clone_for_appearance_output_test();
        let presentation =
            worth_ui_host_contract::UiMountedPresentationAttemptIdentity::mint_unbound().unwrap();
        let before = rejected.pointer.work();
        rejected.lower_appearance(presentation, &[], None);
        assert_eq!(rejected.unpublished_appearance().unwrap_err(),
            &crate::mounting::projection::UiMountedAppearanceOutputDenial::Transport(
                worth_ui_host_contract::UiUnpublishedAppearanceFrameProjectionDenial::SurfaceBindingMismatch));
        assert_eq!(rejected.pointer.work(), before);
        rejected.lower_appearance(presentation, self.manifest.surfaces(), None);
        control.lower_appearance(presentation, self.manifest.surfaces(), None);
        assert_eq!(
            rejected.unpublished_appearance().unwrap(),
            control.unpublished_appearance().unwrap()
        );
        assert!(rejected.unpublished_appearance().unwrap().is_some());
    }

    #[cfg(test)]
    pub(crate) fn lower_reconstructed_pointer_for_test(
        &self,
    ) -> worth_ui_host_contract::UiUnpublishedAppearanceFrameProjection {
        let mut owner = self.candidate.owner.clone_for_appearance_output_test();
        owner.prepare_appearance_reconstruction().unwrap();
        let presentation =
            worth_ui_host_contract::UiMountedPresentationAttemptIdentity::mint_unbound().unwrap();
        owner.lower_appearance(presentation, self.manifest.surfaces(), None);
        owner.unpublished_appearance().unwrap().unwrap().clone()
    }
}
