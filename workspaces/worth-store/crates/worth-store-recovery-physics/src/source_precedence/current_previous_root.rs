use worth_store_physical_format::{
    store_namespace::StableStoreIdentity, CurrentRootCatalogGeneration,
    PhysicalRecordFormatDeclaration,
};

use super::{PhysicalRootSlotObservation, PhysicalRootSourceCandidate};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SelectedPhysicalRoot {
    selected: PhysicalRootSourceCandidate,
    role: SelectedPhysicalRootRole,
    retained_previous: Option<PhysicalRootSourceCandidate>,
    current_rejected: bool,
    previous_rejected: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SelectedPhysicalRootRole {
    Current,
    PreviousFallback,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PhysicalRootSelectionDenial {
    NoAdmittedRoot,
    CurrentRootRejected,
    PreviousFallbackUnlinked,
    PreviousFallbackUnanchored,
    PreviousFallbackAnchorStoreMismatch,
    PreviousFallbackAnchorFormatMismatch,
    PreviousFallbackAnchorGenerationMismatch,
    PreviousFallbackAnchorIdentityMismatch,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PhysicalBootstrapFallbackAnchor {
    store: StableStoreIdentity,
    format: PhysicalRecordFormatDeclaration,
    current_root_generation: CurrentRootCatalogGeneration,
}

impl PhysicalBootstrapFallbackAnchor {
    pub const fn from_integrity_projection(
        store: StableStoreIdentity,
        format: PhysicalRecordFormatDeclaration,
        current_root_generation: CurrentRootCatalogGeneration,
    ) -> Self {
        Self {
            store,
            format,
            current_root_generation,
        }
    }
}

pub fn select_current_previous_root(
    current: PhysicalRootSlotObservation,
    previous: PhysicalRootSlotObservation,
    fallback_anchor: Option<PhysicalBootstrapFallbackAnchor>,
) -> Result<SelectedPhysicalRoot, PhysicalRootSelectionDenial> {
    match current {
        PhysicalRootSlotObservation::Candidate(current) => select_current(current, previous),
        PhysicalRootSlotObservation::SelectorRejected(
            super::PhysicalRootSelectorDenial::Conflict,
        )
        | PhysicalRootSlotObservation::RootRejected { .. } => {
            Err(PhysicalRootSelectionDenial::CurrentRootRejected)
        }
        PhysicalRootSlotObservation::Absent => Err(PhysicalRootSelectionDenial::NoAdmittedRoot),
        PhysicalRootSlotObservation::SelectorRejected(_) => {
            select_previous(previous, fallback_anchor)
        }
    }
}

fn select_current(
    current: PhysicalRootSourceCandidate,
    previous: PhysicalRootSlotObservation,
) -> Result<SelectedPhysicalRoot, PhysicalRootSelectionDenial> {
    let current_selector = current.selector();
    let previous_expected_or_observed = current_selector.linked_selector().is_some()
        || !matches!(&previous, PhysicalRootSlotObservation::Absent);
    let retained_previous = match (current_selector.linked_selector(), previous) {
        (Some(previous_identity), PhysicalRootSlotObservation::Candidate(previous)) => {
            let previous_selector = previous.selector();
            let reciprocal = current_selector.linked_root_generation()
                == Some(previous_selector.root_generation())
                && previous_selector.linked_selector() == Some(current_selector.identity())
                && previous_selector.linked_root_generation()
                    == Some(current_selector.root_generation())
                && previous_selector.identity() == previous_identity;
            reciprocal.then_some(previous)
        }
        _ => None,
    };
    let previous_rejected = retained_previous.is_none() && previous_expected_or_observed;
    Ok(SelectedPhysicalRoot {
        selected: current,
        role: SelectedPhysicalRootRole::Current,
        retained_previous,
        current_rejected: false,
        previous_rejected,
    })
}

fn select_previous(
    previous: PhysicalRootSlotObservation,
    fallback_anchor: Option<PhysicalBootstrapFallbackAnchor>,
) -> Result<SelectedPhysicalRoot, PhysicalRootSelectionDenial> {
    let PhysicalRootSlotObservation::Candidate(previous) = previous else {
        return Err(PhysicalRootSelectionDenial::NoAdmittedRoot);
    };
    let selector = previous.selector();
    let (Some(linked_selector), Some(linked_generation)) = (
        selector.linked_selector(),
        selector.linked_root_generation(),
    ) else {
        return Err(PhysicalRootSelectionDenial::PreviousFallbackUnlinked);
    };
    let Some(anchor) = fallback_anchor else {
        return Err(PhysicalRootSelectionDenial::PreviousFallbackUnanchored);
    };
    if anchor.store != selector.store_identity() {
        return Err(PhysicalRootSelectionDenial::PreviousFallbackAnchorStoreMismatch);
    }
    if anchor.format != selector.format() {
        return Err(PhysicalRootSelectionDenial::PreviousFallbackAnchorFormatMismatch);
    }
    let anchor_generation = anchor.current_root_generation.get();
    if anchor_generation != linked_generation {
        return Err(PhysicalRootSelectionDenial::PreviousFallbackAnchorGenerationMismatch);
    }
    if anchor_generation != linked_selector.get() {
        return Err(PhysicalRootSelectionDenial::PreviousFallbackAnchorIdentityMismatch);
    }
    Ok(SelectedPhysicalRoot {
        selected: previous,
        role: SelectedPhysicalRootRole::PreviousFallback,
        retained_previous: None,
        current_rejected: true,
        previous_rejected: false,
    })
}

impl SelectedPhysicalRoot {
    pub const fn selected(&self) -> &PhysicalRootSourceCandidate {
        &self.selected
    }

    pub const fn role(&self) -> SelectedPhysicalRootRole {
        self.role
    }

    pub const fn retained_previous(&self) -> Option<&PhysicalRootSourceCandidate> {
        self.retained_previous.as_ref()
    }

    pub const fn previous_rejected(&self) -> bool {
        self.previous_rejected
    }

    pub const fn current_rejected(&self) -> bool {
        self.current_rejected
    }
}

#[cfg(test)]
mod tests {
    use worth_store_physical_format::{
        store_namespace::{
            ProposedStoreIdentity, StableStoreIdentity, StoreNamespaceIdentityRecord,
            StoreNamespaceVersion,
        },
        DurablePhysicalRootManifest, DurableRootSelector, FreeSpaceBlockReference, FreeSpaceKey,
        PhysicalRecordFormatDeclaration, RecordAllocationClass, RootSelectorIdentity,
        RootSelectorRole,
    };

    use super::*;
    use crate::{
        observe_structured_physical_root_candidate, PhysicalRootCandidateDenial,
        PhysicalRootManifestDenial, PhysicalRootSelectorDenial, PhysicalRootSlotObservation,
    };

    #[test]
    fn current_role_wins_even_when_previous_has_the_higher_generation() {
        let current = slot(RootSelectorRole::Current, 20, 2, Some((19, 9)));
        let previous = slot(RootSelectorRole::Previous, 19, 9, Some((20, 2)));
        let selected = select_current_previous_root(current, previous, None).unwrap();
        assert_eq!(
            selected.role(),
            SelectedPhysicalRootRole::Current,
            "MUTANT_PREDICATE:c8-current-selector-outranks-higher-generation"
        );
        assert_eq!(selected.selected().manifest().generation(), 2);
        assert_eq!(
            selected
                .retained_previous()
                .unwrap()
                .manifest()
                .generation(),
            9
        );
    }

    #[test]
    fn torn_current_uses_only_the_linked_previous_slot() {
        let current =
            PhysicalRootSlotObservation::SelectorRejected(PhysicalRootSelectorDenial::Integrity);
        let previous = slot(RootSelectorRole::Previous, 1, 1, Some((2, 2)));
        let selected = select_current_previous_root(current, previous, Some(catalog(2))).unwrap();
        assert_eq!(selected.role(), SelectedPhysicalRootRole::PreviousFallback);
        assert_eq!(selected.selected().manifest().generation(), 1);
    }

    #[test]
    fn authority_mismatched_current_preserves_previous_fallback() {
        let current = PhysicalRootSlotObservation::SelectorRejected(
            PhysicalRootSelectorDenial::AuthorityMismatch,
        );
        let previous = slot(RootSelectorRole::Previous, 1, 1, Some((2, 2)));
        let selected = select_current_previous_root(current, previous, Some(catalog(2))).unwrap();
        assert_eq!(selected.role(), SelectedPhysicalRootRole::PreviousFallback);
        assert_eq!(selected.selected().manifest().generation(), 1);
    }

    #[test]
    fn conflicting_current_blocks_even_with_an_exact_anchored_previous() {
        let current =
            PhysicalRootSlotObservation::SelectorRejected(PhysicalRootSelectorDenial::Conflict);
        let previous = slot(RootSelectorRole::Previous, 1, 1, Some((2, 2)));
        assert_eq!(
            select_current_previous_root(current, previous, Some(catalog(2))),
            Err(PhysicalRootSelectionDenial::CurrentRootRejected)
        );
    }

    #[test]
    fn absent_current_slot_never_promotes_a_previous_selector() {
        let previous = slot(RootSelectorRole::Previous, 19, 1, Some((20, 2)));
        assert_eq!(
            select_current_previous_root(PhysicalRootSlotObservation::Absent, previous, None),
            Err(PhysicalRootSelectionDenial::NoAdmittedRoot)
        );
    }

    #[test]
    fn valid_current_selector_with_missing_root_never_demotes_to_previous() {
        let current_selector = selector(RootSelectorRole::Current, 20, 2, Some((19, 1)));
        let current = PhysicalRootSlotObservation::RootRejected {
            denial: PhysicalRootManifestDenial::Integrity,
            selector: current_selector,
        };
        let previous = slot(RootSelectorRole::Previous, 19, 1, Some((20, 2)));
        assert_eq!(
            select_current_previous_root(current, previous, None),
            Err(PhysicalRootSelectionDenial::CurrentRootRejected)
        );
    }

    #[test]
    fn mismatched_previous_is_rejected_without_overriding_a_valid_current_root() {
        let current = slot(RootSelectorRole::Current, 20, 2, Some((19, 1)));
        let previous = slot(RootSelectorRole::Previous, 19, 1, Some((20, 3)));
        let selected = select_current_previous_root(current, previous, None).unwrap();
        assert_eq!(selected.role(), SelectedPhysicalRootRole::Current);
        assert!(selected.retained_previous().is_none());
        assert!(selected.previous_rejected());
    }

    #[test]
    fn root_generation_mismatch_is_rejected() {
        let selector = selector(RootSelectorRole::Current, 20, 2, None);
        let observed = observe_structured_physical_root_candidate(selector, manifest(3), format());
        assert_eq!(
            observed.rejection().map(|(denial, _)| denial),
            Some(PhysicalRootCandidateDenial::RootGenerationMismatch),
            "MUTANT_PREDICATE:c8-root-generation-binding-ignored"
        );
    }

    #[test]
    fn previous_only_publication_prefix_keeps_the_old_current_authoritative() {
        let old_current = slot(RootSelectorRole::Current, 10, 1, None);
        let newly_published_previous = slot(RootSelectorRole::Previous, 19, 1, Some((20, 2)));
        let selected =
            select_current_previous_root(old_current, newly_published_previous, None).unwrap();
        assert_eq!(selected.role(), SelectedPhysicalRootRole::Current);
        assert_eq!(selected.selected().manifest().generation(), 1);
        assert!(selected.previous_rejected());
    }

    fn slot(
        role: RootSelectorRole,
        identity: u64,
        generation: u64,
        linked: Option<(u64, u64)>,
    ) -> PhysicalRootSlotObservation {
        let selector = selector(role, identity, generation, linked);
        let format = format();
        observe_structured_physical_root_candidate(selector, manifest(generation), format)
    }

    fn selector(
        role: RootSelectorRole,
        identity: u64,
        generation: u64,
        linked: Option<(u64, u64)>,
    ) -> DurableRootSelector {
        selector_for_store(store(), role, identity, generation, linked)
    }

    fn selector_for_store(
        store: StableStoreIdentity,
        role: RootSelectorRole,
        identity: u64,
        generation: u64,
        linked: Option<(u64, u64)>,
    ) -> DurableRootSelector {
        DurableRootSelector::new(
            store,
            format(),
            RootSelectorIdentity::new(identity).unwrap(),
            role,
            generation,
            linked.and_then(|(identity, _)| RootSelectorIdentity::new(identity)),
            linked.map(|(_, generation)| generation),
        )
        .unwrap()
    }

    fn manifest(generation: u64) -> DurablePhysicalRootManifest {
        let key = FreeSpaceKey::new(RecordAllocationClass::Extent, 1).unwrap();
        let free = FreeSpaceBlockReference::new(generation, 1, 0, 17, key, key).unwrap();
        DurablePhysicalRootManifest::builder(generation, 7, 4, 19)
            .free_space_root(Some(free))
            .admit()
            .unwrap()
    }

    fn format() -> PhysicalRecordFormatDeclaration {
        PhysicalRecordFormatDeclaration::builder().admit().unwrap()
    }

    fn catalog(generation: u64) -> PhysicalBootstrapFallbackAnchor {
        PhysicalBootstrapFallbackAnchor::from_integrity_projection(
            store(),
            format(),
            CurrentRootCatalogGeneration::new(generation).unwrap(),
        )
    }

    fn store() -> StableStoreIdentity {
        StoreNamespaceIdentityRecord::new(
            StoreNamespaceVersion::CURRENT,
            ProposedStoreIdentity::from_nonzero_bytes([7; 16]).unwrap(),
        )
        .published_identity()
    }
}
