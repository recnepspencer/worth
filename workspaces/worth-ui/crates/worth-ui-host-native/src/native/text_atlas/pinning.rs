//! Move-only live-layout pin authority and copyable diagnostic snapshots.

use std::collections::HashSet;

use worth_ui_host_contract::{UiGlyphRasterKey, UiQualifiedTextLayoutIdentity};

use super::key::UiAtlasEntryIdentity;
use super::ownership::{AtlasCore, PinIdentity};
use super::recovery::{UiNativeTextAtlasDenial, UiNativeTextAtlasGeneration};
use super::transaction::UiNativeTextAtlasPinTransition;
use super::UiNativeTextAtlasDemand;

pub(crate) struct UiNativeTextAtlasPin {
    layout: UiQualifiedTextLayoutIdentity,
    key: UiGlyphRasterKey,
    entry: UiAtlasEntryIdentity,
    generation: UiNativeTextAtlasGeneration,
}

impl UiNativeTextAtlasPin {
    pub(crate) const fn from_native_host(
        layout: UiQualifiedTextLayoutIdentity,
        key: UiGlyphRasterKey,
        entry: UiAtlasEntryIdentity,
        generation: UiNativeTextAtlasGeneration,
    ) -> Self {
        Self {
            layout,
            key,
            entry,
            generation,
        }
    }

    pub(crate) fn identity(&self) -> PinIdentity {
        PinIdentity::new(self.layout, self.key)
    }

    pub(crate) const fn layout(&self) -> UiQualifiedTextLayoutIdentity {
        self.layout
    }

    pub(crate) const fn key(&self) -> UiGlyphRasterKey {
        self.key
    }

    pub(crate) const fn entry(&self) -> UiAtlasEntryIdentity {
        self.entry
    }

    pub(crate) const fn generation(&self) -> UiNativeTextAtlasGeneration {
        self.generation
    }
}

pub(crate) fn protected_keys(
    core: &AtlasCore,
    transition: &UiNativeTextAtlasPinTransition,
    demands: &[UiNativeTextAtlasDemand],
) -> HashSet<UiGlyphRasterKey> {
    let mut keys = demands
        .iter()
        .map(|demand| demand.key())
        .collect::<HashSet<_>>();
    let released = transition
        .releases()
        .iter()
        .map(|release| PinIdentity::new(release.layout(), release.key()))
        .collect::<HashSet<_>>();
    for pin in core
        .pins
        .values()
        .filter(|pin| !released.contains(&pin.identity()))
    {
        keys.insert(pin.key());
    }
    for addition in transition.additions() {
        keys.insert(addition.key());
    }
    // The demand records are the plan's work; the pins are this stage's.
    // Releases are walked but not inserted: validation proved each is pinned.
    let inserted = core.pins.len() + transition.additions().len();
    worth_ui_host_contract::record_presentation_glyphs(
        worth_ui_host_contract::UiPresentationWorkStage::Pins,
        transition.releases().len() + inserted,
    );
    worth_ui_host_contract::record_presentation_map_inserts(demands.len() + inserted);
    keys
}

pub(crate) fn validate_pin_transition(
    core: &AtlasCore,
    transition: &UiNativeTextAtlasPinTransition,
) -> Result<(), UiNativeTextAtlasDenial> {
    let transitions = transition.releases().len() + transition.additions().len();
    worth_ui_host_contract::record_presentation_glyphs(
        worth_ui_host_contract::UiPresentationWorkStage::Pins,
        transitions,
    );
    worth_ui_host_contract::record_presentation_map_inserts(transitions);
    let mut seen = HashSet::new();
    for release in transition.releases() {
        if !seen.insert(PinIdentity::new(release.layout(), release.key()))
            || !core
                .pins
                .contains_key(&PinIdentity::new(release.layout(), release.key()))
        {
            return Err(UiNativeTextAtlasDenial::StalePin);
        }
    }
    for add in transition.additions() {
        if !seen.insert(PinIdentity::new(add.layout(), add.key()))
            || core
                .pins
                .contains_key(&PinIdentity::new(add.layout(), add.key()))
        {
            return Err(UiNativeTextAtlasDenial::PinConflict);
        }
    }
    Ok(())
}
