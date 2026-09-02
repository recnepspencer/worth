//! Runtime-owned command-scoped text pin candidates and commit authority.

use std::collections::{BTreeMap, HashMap};

use worth_ui_host_contract::{
    UiGlyphRasterPinRequest, UiMountedPaintCommandIdentity, UiSurfaceBindingGeneration,
};

use crate::native_platform::text_presentation::UiNativeTextPresentationPrepared;

#[derive(Clone, Default, Eq, PartialEq)]
struct UiMountedBindingPins {
    by_command: HashMap<UiMountedPaintCommandIdentity, Box<[UiGlyphRasterPinRequest]>>,
    pin_owners: HashMap<UiGlyphRasterPinRequest, u32>,
}

#[derive(Default)]
pub(crate) struct UiMountedTextPinState {
    committed: BTreeMap<UiSurfaceBindingGeneration, UiMountedBindingPins>,
    global_pin_owners: HashMap<UiGlyphRasterPinRequest, u32>,
}

struct UiMountedTextPinEdit {
    command: UiMountedPaintCommandIdentity,
    pins: Option<Box<[UiGlyphRasterPinRequest]>>,
}

pub(crate) struct UiMountedTextPinCandidate {
    binding: UiSurfaceBindingGeneration,
    next_binding: UiMountedBindingPins,
    #[cfg(test)]
    binding_changed: bool,
    binding_additions: Box<[UiGlyphRasterPinRequest]>,
    binding_releases: Box<[UiGlyphRasterPinRequest]>,
    additions: Box<[UiGlyphRasterPinRequest]>,
    releases: Box<[UiGlyphRasterPinRequest]>,
    binding_pins: Box<[UiGlyphRasterPinRequest]>,
}

impl UiMountedTextPinState {
    pub(crate) fn candidate(
        &self,
        binding: UiSurfaceBindingGeneration,
        prepared: &UiNativeTextPresentationPrepared,
    ) -> UiMountedTextPinCandidate {
        let previous = self.committed.get(&binding).cloned().unwrap_or_default();
        let edits = prepared_pin_edits(prepared);
        let next_binding = projected_binding(previous.clone(), prepared.pin_set_complete(), &edits);
        self.candidate_from_next(binding, previous, next_binding)
    }

    pub(crate) fn deregistration_candidate(
        &self,
        binding: UiSurfaceBindingGeneration,
    ) -> UiMountedTextPinCandidate {
        let previous = self.committed.get(&binding).cloned().unwrap_or_default();
        self.candidate_from_next(binding, previous, UiMountedBindingPins::default())
    }

    fn candidate_from_next(
        &self,
        binding: UiSurfaceBindingGeneration,
        previous: UiMountedBindingPins,
        next_binding: UiMountedBindingPins,
    ) -> UiMountedTextPinCandidate {
        #[cfg(test)]
        let binding_changed = previous.by_command != next_binding.by_command;
        let (binding_additions, binding_releases) = transition_difference(
            &all_pins(&previous).collect::<Vec<_>>(),
            &all_pins(&next_binding).collect::<Vec<_>>(),
        );
        let additions = binding_additions
            .iter()
            .copied()
            .filter(|pin| !self.global_pin_owners.contains_key(pin))
            .collect::<Vec<_>>()
            .into_boxed_slice();
        let releases = binding_releases
            .iter()
            .copied()
            .filter(|pin| self.global_pin_owners.get(pin).copied() == Some(1))
            .collect::<Vec<_>>()
            .into_boxed_slice();
        let binding_pins = all_pins(&next_binding)
            .collect::<Vec<_>>()
            .into_boxed_slice();
        UiMountedTextPinCandidate {
            binding,
            next_binding,
            #[cfg(test)]
            binding_changed,
            binding_additions,
            binding_releases,
            additions,
            releases,
            binding_pins,
        }
    }

    pub(crate) fn commit_presented(&mut self, candidate: UiMountedTextPinCandidate) {
        remove_pin_owners(&mut self.global_pin_owners, &candidate.binding_releases);
        add_pin_owners(&mut self.global_pin_owners, &candidate.binding_additions);
        if candidate.next_binding.by_command.is_empty() {
            self.committed.remove(&candidate.binding);
        } else {
            self.committed
                .insert(candidate.binding, candidate.next_binding);
        }
    }

    pub(crate) fn transition_view(
        candidate: &UiMountedTextPinCandidate,
    ) -> worth_ui_host_contract::UiGlyphRasterPinTransitionView<'_> {
        worth_ui_host_contract::UiGlyphRasterPinTransitionView::from_text_mechanics(
            &candidate.additions,
            &candidate.releases,
        )
    }

    pub(crate) fn binding_pins(
        candidate: &UiMountedTextPinCandidate,
    ) -> &[UiGlyphRasterPinRequest] {
        &candidate.binding_pins
    }

    #[cfg(test)]
    fn committed(&self, binding: UiSurfaceBindingGeneration) -> Vec<UiGlyphRasterPinRequest> {
        self.committed
            .get(&binding)
            .into_iter()
            .flat_map(all_pins)
            .collect()
    }
}

impl UiMountedTextPinCandidate {
    pub(crate) fn has_no_pin_churn(&self) -> bool {
        self.additions.is_empty() && self.releases.is_empty()
    }

    #[cfg(test)]
    pub(crate) const fn changes_binding(&self) -> bool {
        self.binding_changed
    }
}

fn prepared_pin_edits(prepared: &UiNativeTextPresentationPrepared) -> Vec<UiMountedTextPinEdit> {
    let mut edits = prepared
        .pin_commands()
        .iter()
        .copied()
        .zip(prepared.demand_batches())
        .map(|(command, demand)| UiMountedTextPinEdit {
            command,
            pins: Some(
                demand
                    .records()
                    .iter()
                    .map(|record| {
                        UiGlyphRasterPinRequest::from_text_mechanics(
                            demand.layout_identity(),
                            record.key(),
                        )
                    })
                    .collect::<Vec<_>>()
                    .into_boxed_slice(),
            ),
        })
        .collect::<Vec<_>>();
    edits.extend(
        prepared
            .pin_removals()
            .iter()
            .copied()
            .map(|command| UiMountedTextPinEdit {
                command,
                pins: None,
            }),
    );
    edits
}

fn projected_binding(
    mut state: UiMountedBindingPins,
    replace_complete_set: bool,
    edits: &[UiMountedTextPinEdit],
) -> UiMountedBindingPins {
    if replace_complete_set {
        state = UiMountedBindingPins::default();
    }
    for edit in edits {
        if let Some(previous) = state.by_command.remove(&edit.command) {
            remove_pin_owners(&mut state.pin_owners, &previous);
        }
        if let Some(pins) = edit.pins.as_ref().filter(|pins| !pins.is_empty()) {
            add_pin_owners(&mut state.pin_owners, pins);
            state.by_command.insert(edit.command, pins.clone());
        }
    }
    state
}

fn transition_difference(
    previous: &[UiGlyphRasterPinRequest],
    current: &[UiGlyphRasterPinRequest],
) -> (
    Box<[UiGlyphRasterPinRequest]>,
    Box<[UiGlyphRasterPinRequest]>,
) {
    (
        current
            .iter()
            .copied()
            .filter(|pin| !previous.contains(pin))
            .collect::<Vec<_>>()
            .into_boxed_slice(),
        previous
            .iter()
            .copied()
            .filter(|pin| !current.contains(pin))
            .collect::<Vec<_>>()
            .into_boxed_slice(),
    )
}

fn all_pins(state: &UiMountedBindingPins) -> impl Iterator<Item = UiGlyphRasterPinRequest> + '_ {
    state.pin_owners.keys().copied()
}

fn add_pin_owners(
    counts: &mut HashMap<UiGlyphRasterPinRequest, u32>,
    pins: &[UiGlyphRasterPinRequest],
) {
    for pin in pins {
        let count = counts.entry(*pin).or_default();
        *count = count.saturating_add(1);
    }
}

fn remove_pin_owners(
    counts: &mut HashMap<UiGlyphRasterPinRequest, u32>,
    pins: &[UiGlyphRasterPinRequest],
) {
    for pin in pins {
        if let Some(count) = counts.get_mut(pin) {
            *count = count.saturating_sub(1);
            if *count == 0 {
                counts.remove(pin);
            }
        }
    }
}

impl UiMountedTextPinCandidate {
    #[cfg(test)]
    fn additions(&self) -> &[UiGlyphRasterPinRequest] {
        &self.additions
    }

    #[cfg(test)]
    fn releases(&self) -> &[UiGlyphRasterPinRequest] {
        &self.releases
    }
}

#[cfg(test)]
#[path = "text_pins_tests.rs"]
mod tests;
