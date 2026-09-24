//! Certification-selected destruction applied only by the native state owners.

use std::collections::{BTreeSet, HashSet};

use crate::UiNativeDerivedStateLossClass as LossClass;

use super::UiNativeHostState;

impl UiNativeHostState {
    pub(crate) fn apply_completed_qualified_derived_state_loss(&mut self, binding: u64) {
        let completed = self.retained_frame_observations.len() as u64;
        let Some(class) = self.qualification.completed_derived_state_loss(completed) else {
            return;
        };
        if !self.can_apply_qualified_derived_state_loss(binding, class) {
            return;
        }
        self.qualification.commit_completed_derived_state_loss();
        self.apply_qualified_derived_state_loss(binding, class);
    }

    pub(crate) fn can_apply_qualified_derived_state_loss(
        &self,
        binding: u64,
        class: LossClass,
    ) -> bool {
        match class {
            LossClass::TextAtlasPagesAndIndex => {
                self.text_pins_by_binding.len() == 1
                    && self.text_pins_by_binding.contains_key(&binding)
                    && self.text_atlas.can_mutate_for_reconstruction()
                    && self
                        .text_atlas_gpu
                        .as_ref()
                        .is_none_or(|gpu| gpu.can_close())
            }
            LossClass::TextAtlasPins => {
                self.text_pins_by_binding.contains_key(&binding)
                    && self.text_atlas.can_mutate_for_reconstruction()
            }
            LossClass::RetainedDrawList => self.retained_draw_lists.contains_key(&binding),
            LossClass::RetainedTarget => {
                self.retained_draw_lists.len() == 1
                    && self.retained_draw_lists.contains_key(&binding)
                    && self.presentation_owners.is_some()
                    && self.resources.admits(1)
            }
            LossClass::PresentationAffinity => {
                self.presentation_epochs.contains_key(&binding)
                    || self
                        .last_retained_frame
                        .as_ref()
                        .is_some_and(|presentation| presentation.binding_generation() == binding)
            }
        }
    }

    pub(crate) fn apply_qualified_derived_state_loss(&mut self, binding: u64, class: LossClass) {
        assert!(self.can_apply_qualified_derived_state_loss(binding, class));
        let affected = match class {
            LossClass::TextAtlasPagesAndIndex => self.destroy_text_atlas(binding),
            LossClass::TextAtlasPins => self.destroy_text_pins(binding),
            LossClass::RetainedDrawList => {
                self.retained_draw_lists
                    .remove(&binding)
                    .expect("preflight retained draw list");
                BTreeSet::from([binding])
            }
            LossClass::RetainedTarget => self.destroy_retained_target(binding),
            LossClass::PresentationAffinity => {
                self.presentation_epochs.remove(&binding);
                if self
                    .last_retained_frame
                    .as_ref()
                    .is_some_and(|presentation| presentation.binding_generation() == binding)
                {
                    self.last_retained_frame = None;
                }
                BTreeSet::from([binding])
            }
        };
        for binding in &affected {
            self.captures.invalidate_source(*binding);
        }
        for binding in affected.iter().copied() {
            self.lifecycle.require_recovery(
                binding,
                crate::native::UiNativeRecoveryCause::DerivedStateLost,
            );
        }
        self.qualification
            .record_derived_state_loss(class, affected);
    }

    pub(crate) fn record_qualified_derived_state_reconstruction(&mut self, binding: u64) {
        let Some(class) = self.qualification.pending_reconstruction() else {
            return;
        };
        let restored = match class {
            LossClass::TextAtlasPagesAndIndex => self.text_atlas_is_live_for(binding),
            LossClass::TextAtlasPins => self.text_pins_are_live_for(binding),
            LossClass::RetainedDrawList => self.retained_draw_lists.contains_key(&binding),
            LossClass::RetainedTarget => {
                self.presentation_access().is_some_and(|access| {
                    let _ = access.retained_target();
                    true
                }) && self.presentation_affinity_is_live_for(binding)
            }
            LossClass::PresentationAffinity => self.presentation_affinity_is_live_for(binding),
        };
        self.qualification
            .record_derived_state_reconstruction(binding, restored);
    }

    fn destroy_text_atlas(&mut self, binding: u64) -> BTreeSet<u64> {
        let mut affected = self
            .text_pins_by_binding
            .keys()
            .copied()
            .collect::<BTreeSet<_>>();
        affected.insert(binding);
        if let Some(gpu) = self.text_atlas_gpu.take() {
            gpu.try_close(&mut self.resources)
                .unwrap_or_else(|_| panic!("preflight proved atlas GPU pages closable"));
        }
        assert!(self.text_atlas.clear(), "preflight proved atlas mutable");
        self.text_pins_by_binding.clear();
        affected
    }

    fn destroy_text_pins(&mut self, binding: u64) -> BTreeSet<u64> {
        let requests = self
            .text_pins_by_binding
            .get(&binding)
            .expect("preflight retained binding pins");
        let shared = self
            .text_pins_by_binding
            .iter()
            .filter(|(candidate, _)| **candidate != binding)
            .flat_map(|(_, requests)| requests.iter().copied())
            .collect::<HashSet<_>>();
        let exclusive = requests
            .iter()
            .copied()
            .filter(|request| !shared.contains(request))
            .collect::<Vec<_>>();
        assert!(
            self.text_atlas.remove_pins(&exclusive),
            "preflight proved atlas pins mutable"
        );
        self.text_pins_by_binding.remove(&binding);
        BTreeSet::from([binding])
    }

    fn destroy_retained_target(&mut self, binding: u64) -> BTreeSet<u64> {
        let mut affected = self
            .retained_draw_lists
            .keys()
            .copied()
            .collect::<BTreeSet<_>>();
        affected.insert(binding);
        let Some(crate::native::UiNativePresentationOwners { device, surface }) =
            self.presentation_owners.as_mut()
        else {
            panic!("preflight retained presentation owners");
        };
        crate::native::lifecycle::replace_retained_target_for_reconstruction(
            device,
            surface,
            &mut self.resources,
        )
        .expect("preflight reserved one replacement target owner");
        affected
    }

    fn text_atlas_is_live_for(&self, binding: u64) -> bool {
        let snapshot = self.text_atlas.snapshot();
        let gpu_pages = self.text_atlas_gpu.as_ref().map_or(0, |gpu| {
            gpu.page_count(crate::native::text_atlas::UiNativeGpuAtlasKind::Alpha)
                + gpu.page_count(crate::native::text_atlas::UiNativeGpuAtlasKind::Color)
        });
        snapshot.alpha_entries + snapshot.color_entries > 0
            && gpu_pages > 0
            && self.text_pins_are_live_for(binding)
    }

    fn text_pins_are_live_for(&self, binding: u64) -> bool {
        let Some(requests) = self.text_pins_by_binding.get(&binding) else {
            return false;
        };
        let observed = self.text_atlas.pin_observations();
        !requests.is_empty()
            && requests
                .iter()
                .all(|request| observed.iter().any(|pin| pin.matches(*request)))
    }

    fn presentation_affinity_is_live_for(&self, binding: u64) -> bool {
        self.presentation_epochs.contains_key(&binding)
            && self
                .last_retained_frame
                .as_ref()
                .is_some_and(|presentation| presentation.binding_generation() == binding)
    }
}
