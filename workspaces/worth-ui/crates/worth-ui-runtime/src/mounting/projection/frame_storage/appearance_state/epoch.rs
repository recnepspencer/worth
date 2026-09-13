use super::{UiMountedAppearanceEpoch, UiMountedAppearanceFrameState};

impl UiMountedAppearanceFrameState {
    pub(crate) fn requires_epoch_transition(
        &self,
        session: crate::facade::WorthUiActiveApplicationSessionIdentity,
        generation: &crate::runtime::WorthUiActiveApplicationGenerationIdentity,
    ) -> bool {
        self.epoch
            .as_ref()
            .is_none_or(|epoch| epoch.session != session || epoch.generation != *generation)
    }

    pub(crate) fn begin_epoch(
        &mut self,
        session: crate::facade::WorthUiActiveApplicationSessionIdentity,
        generation: &crate::runtime::WorthUiActiveApplicationGenerationIdentity,
        retired_instances: &[worth_ui_host_contract::UiMountedInstanceIdentity],
    ) -> usize {
        let epoch = UiMountedAppearanceEpoch {
            session,
            generation: generation.clone(),
        };
        let mut retired_memberships = 0;
        if self.epoch.as_ref() != Some(&epoch) {
            let (retired, work) = self.members.clear_for_epoch();
            retired_memberships = retired;
            self.selection.record_membership_work(work);
            self.epoch = Some(epoch);
            self.capacity_error = None;
        }
        let (retired, work) = self
            .members
            .retire_instances(retired_instances, &mut self.retirements);
        self.selection.record_membership_work(work);
        retired_memberships.saturating_add(retired)
    }

    pub(in crate::mounting::projection::frame_storage) fn admits_retained_generation(
        &self,
        owners: &crate::runtime::appearance::UiPreparedRetainedAppearanceOwnerSuccession,
    ) -> bool {
        self.epoch.as_ref().is_none_or(|epoch| {
            owners.generations().is_none_or(|(prior, _)| {
                epoch.session == prior.session_identity() && epoch.generation == *prior
            })
        })
    }

    pub(in crate::mounting::projection::frame_storage) fn commit_retained_generation(
        &mut self,
        owners: &crate::runtime::appearance::UiPreparedRetainedAppearanceOwnerSuccession,
    ) {
        assert!(
            self.admits_retained_generation(owners),
            "retained appearance epoch was admitted before effects"
        );
        if let (Some(epoch), Some((_, successor))) = (self.epoch.as_mut(), owners.generations()) {
            // Exact role, theme, consumer and owner equivalence preserves the cached
            // semantic projection. Its original resolution evidence stays immutable.
            epoch.generation = successor.clone();
        }
    }

    pub(crate) fn retire_detached_on_epoch_change(
        &mut self,
        session: crate::facade::WorthUiActiveApplicationSessionIdentity,
        generation: &crate::runtime::WorthUiActiveApplicationGenerationIdentity,
        attached_instances: &std::collections::BTreeSet<
            worth_ui_host_contract::UiMountedInstanceIdentity,
        >,
    ) -> usize {
        if !self.requires_epoch_transition(session, generation) {
            return 0;
        }
        let (retired, work) = self
            .members
            .retire_unattached(attached_instances, &mut self.retirements);
        self.selection.record_membership_work(work);
        retired
    }
}
