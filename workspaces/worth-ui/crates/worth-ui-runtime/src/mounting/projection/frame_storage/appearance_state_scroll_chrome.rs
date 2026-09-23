//! Scroll chrome that must be lowered this attempt, and the fragments that
//! carry chrome for owners which paint nothing of their own.
//!
//! Chrome is painted in the fragment of the Scroll region occurrence that
//! reserved its gutter. When that occurrence has an appearance of its own the
//! chrome rides in its node fragment, and a bar that moved, changed posture or
//! changed theme means that node lowers again although nothing about the node
//! itself changed. Most scroll owners are plain containers with no appearance
//! at all; for those the chrome is the whole fragment, and this file keeps one
//! sidecar per such owner so a moved bar becomes a delta against the bar it
//! replaces and a vanished bar becomes a removal. Either way an owner whose
//! chrome is unchanged stays retained, which keeps a frame with a stationary
//! bar on the no-work path.

use std::collections::{BTreeMap, BTreeSet};

use worth_ui_host_contract::{
    UiMountedFrameIdentity, UiMountedInstanceIdentity, UiMountedNodeReceiptIdentity,
    UiMountedPresentationAttemptIdentity, UiMountedSurfaceBindingRequirement,
    UiSemanticSurfaceIdentity,
};

use super::super::appearance_output::{
    UiMountedAppearanceNodeWork, UiMountedAppearanceOutputDenial,
};
use super::super::{UiMountedAppearanceNodeInputContext, UiMountedProjectionFrame};
use super::UiMountedAppearanceFrameState;
use crate::mounting::projection::appearance::{
    UiMountedAppearanceGeometryScope, UiMountedAppearanceLoweringInput,
    UiMountedAppearanceScrollChromeInput, UiMountedAppearanceSidecar,
};

/// The fragment an owner without an appearance of its own presents its chrome in.
#[derive(Clone)]
struct UiScrollChromeOwnerFragment {
    sidecar: UiMountedAppearanceSidecar,
    semantic_surface: UiSemanticSurfaceIdentity,
    /// The receipt the fragment was last issued under; the next issue names it
    /// as the predecessor so the host replaces rather than duplicates.
    receipt: Option<UiMountedNodeReceiptIdentity>,
}

/// One owner whose chrome-only fragment must be issued this attempt.
#[derive(Clone)]
enum UiScrollChromeOwnerWork {
    /// Lower the owner's current chrome as its whole fragment.
    Present {
        context: UiMountedAppearanceNodeInputContext,
        reconstruct: bool,
    },
    /// The owner presents no chrome any more, left the frame, or now paints
    /// its chrome inside a node fragment: remove the chrome-only fragment.
    Absent(UiMountedInstanceIdentity),
}

#[derive(Clone, Default)]
pub(super) struct UiMountedAppearanceScrollChromeState {
    /// The chrome each owner presented last attempt, the basis for deciding
    /// which owners must lower again.
    by_owner: BTreeMap<UiMountedInstanceIdentity, Vec<UiMountedAppearanceScrollChromeInput>>,
    fragments: BTreeMap<UiMountedInstanceIdentity, UiScrollChromeOwnerFragment>,
    /// This attempt's staged owner work. It is never inherited.
    staged: Vec<UiScrollChromeOwnerWork>,
}

impl UiMountedAppearanceScrollChromeState {
    pub(super) fn fork(&self) -> Self {
        Self {
            by_owner: self.by_owner.clone(),
            fragments: self.fragments.clone(),
            staged: Vec::new(),
        }
    }

    pub(super) fn has_pending(&self) -> bool {
        !self.staged.is_empty()
    }

    pub(super) fn clear_for_epoch(&mut self) {
        *self = Self::default();
    }
}

fn bound(
    bindings: &[UiMountedSurfaceBindingRequirement],
    surface: UiSemanticSurfaceIdentity,
) -> bool {
    bindings
        .iter()
        .any(|binding| binding.semantic_surface() == surface)
}

impl UiMountedAppearanceFrameState {
    /// Stage the lowering every owner needs for the chrome it presents this
    /// attempt, and remember that chrome as the basis for the next comparison.
    ///
    /// Chrome on a surface this attempt does not bind is not presented and is
    /// not recorded, so it is compared again once the surface is bound. A
    /// chrome-only fragment whose surface is unbound is likewise kept until it
    /// can be removed on a bound surface.
    pub(in crate::mounting::projection::frame_storage) fn stage_scroll_chrome_changes(
        &mut self,
        frame: &UiMountedProjectionFrame,
        bindings: &[UiMountedSurfaceBindingRequirement],
        chrome: &[UiMountedAppearanceScrollChromeInput],
    ) -> Result<(), UiMountedAppearanceOutputDenial> {
        let mut next: BTreeMap<
            UiMountedInstanceIdentity,
            Vec<UiMountedAppearanceScrollChromeInput>,
        > = BTreeMap::new();
        for input in chrome
            .iter()
            .filter(|input| bound(bindings, input.semantic_surface()))
        {
            next.entry(input.owner_instance()).or_default().push(*input);
        }
        let reconstruct = self.reconstruction_nodes.is_some();
        let owners = self
            .scroll_chrome
            .by_owner
            .keys()
            .chain(next.keys())
            .chain(self.scroll_chrome.fragments.keys())
            .copied()
            .collect::<BTreeSet<_>>();
        for owner in owners {
            let fragment = self.scroll_chrome.fragments.get(&owner);
            let fragment_removable =
                fragment.is_some_and(|fragment| bound(bindings, fragment.semantic_surface));
            let previous_bound = self
                .scroll_chrome
                .by_owner
                .get(&owner)
                .is_some_and(|inputs| {
                    inputs
                        .iter()
                        .any(|input| bound(bindings, input.semantic_surface()))
                });
            // An exact-surface attempt says nothing about retained bars on
            // another surface. Absence from this candidate is not removal.
            if !previous_bound && !next.contains_key(&owner) && !fragment_removable {
                continue;
            }
            let changed = self.scroll_chrome.by_owner.get(&owner) != next.get(&owner);
            let lingering = fragment.is_some() && !next.contains_key(&owner);
            let refreshed = changed || lingering || (reconstruct && fragment.is_some());
            if !refreshed {
                continue;
            }
            let Ok(context) = frame.appearance_node_input(owner) else {
                // The owner left the frame. An owner that painted retires its
                // node fragment on its own path; a chrome-only fragment is
                // retired here.
                if fragment_removable {
                    self.scroll_chrome
                        .staged
                        .push(UiScrollChromeOwnerWork::Absent(owner));
                }
                continue;
            };
            let attached = frame
                .appearance_attached(owner)
                .map_err(|_| UiMountedAppearanceOutputDenial::CurrentProjectionUnavailable)?;
            if attached {
                // The owner paints, so its node fragment carries the chrome: a
                // retained node lowers again, a pending one attaches the chrome
                // when it first lowers.
                if changed {
                    self.stage_input_refresh(&context);
                }
                if fragment_removable {
                    self.scroll_chrome
                        .staged
                        .push(UiScrollChromeOwnerWork::Absent(owner));
                }
            } else if next.contains_key(&owner) {
                self.scroll_chrome
                    .staged
                    .push(UiScrollChromeOwnerWork::Present {
                        context,
                        reconstruct,
                    });
            } else if fragment_removable {
                self.scroll_chrome
                    .staged
                    .push(UiScrollChromeOwnerWork::Absent(owner));
            }
        }
        self.scroll_chrome.by_owner.retain(|_, inputs| {
            !inputs
                .iter()
                .any(|input| bound(bindings, input.semantic_surface()))
        });
        self.scroll_chrome.by_owner.extend(next);
        Ok(())
    }

    /// Issue the chrome-only fragments staged for this attempt: one mount,
    /// reconstruction or removal per owner, each recorded as node work under
    /// the owner's receipt.
    pub(in crate::mounting::projection::frame_storage) fn lower_scroll_chrome_owners(
        &mut self,
        frame: UiMountedFrameIdentity,
        presentation: UiMountedPresentationAttemptIdentity,
        geometry: &UiMountedAppearanceGeometryScope,
    ) -> Result<(), UiMountedAppearanceOutputDenial> {
        for work in std::mem::take(&mut self.scroll_chrome.staged) {
            match work {
                UiScrollChromeOwnerWork::Present {
                    context,
                    reconstruct,
                } => {
                    let chrome = geometry
                        .owned_scroll_chrome(context.mounted_instance)
                        .collect::<Vec<_>>();
                    if chrome.is_empty() {
                        self.remove_scroll_chrome_fragment(
                            context.mounted_instance,
                            frame,
                            presentation,
                        )?;
                        continue;
                    }
                    if !geometry.includes_surface(context.semantic_surface) {
                        return Err(UiMountedAppearanceOutputDenial::CurrentProjectionUnavailable);
                    }
                    let input = UiMountedAppearanceLoweringInput::for_scroll_chrome_owner(
                        context.frame,
                        context.semantic_surface,
                        presentation,
                        context.mounted_instance,
                        chrome,
                    );
                    let fragment = self
                        .scroll_chrome
                        .fragments
                        .entry(context.mounted_instance)
                        .or_insert_with(|| UiScrollChromeOwnerFragment {
                            sidecar: UiMountedAppearanceSidecar::default(),
                            semantic_surface: context.semantic_surface,
                            receipt: None,
                        });
                    let predecessor = fragment.receipt;
                    let lowered = if reconstruct && fragment.sidecar.has_current() {
                        fragment.sidecar.reconstruct(input)
                    } else {
                        fragment.sidecar.mount(input)
                    };
                    let work =
                        lowered.map_err(|_| UiMountedAppearanceOutputDenial::NodeLowering)?;
                    fragment.receipt = Some(context.node_receipt);
                    fragment.semantic_surface = context.semantic_surface;
                    self.node_work.push(UiMountedAppearanceNodeWork {
                        predecessor,
                        successor: Some(context.node_receipt),
                        work,
                    });
                }
                UiScrollChromeOwnerWork::Absent(owner) => {
                    self.remove_scroll_chrome_fragment(owner, frame, presentation)?;
                }
            }
        }
        Ok(())
    }

    fn remove_scroll_chrome_fragment(
        &mut self,
        owner: UiMountedInstanceIdentity,
        frame: UiMountedFrameIdentity,
        presentation: UiMountedPresentationAttemptIdentity,
    ) -> Result<(), UiMountedAppearanceOutputDenial> {
        let Some(fragment) = self.scroll_chrome.fragments.remove(&owner) else {
            return Ok(());
        };
        let work = fragment
            .sidecar
            .removal_work(frame, presentation)
            .map_err(|_| UiMountedAppearanceOutputDenial::NodeLowering)?;
        self.node_work.push(UiMountedAppearanceNodeWork {
            predecessor: fragment.receipt,
            successor: None,
            work,
        });
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn source_generation_transition_retains_scroll_chrome_predecessor() {
        let (session, _, _, _, _) = crate::runtime::appearance::projection_test_inputs();
        let successor = crate::runtime::tests::appearance_component_session_test_support::
            source_backed_appearance_consumer_session();
        let session_identity = session.session_identity();
        let first = session.active_generation_identity();
        let next = crate::runtime::WorthUiActiveApplicationGenerationIdentity::current(
            session_identity,
            successor.generation_identity(),
        );
        let owner = UiMountedInstanceIdentity::mint_unbound().unwrap();
        let mut state = UiMountedAppearanceFrameState::default();
        state.begin_epoch(session_identity, &first, &[]);
        state.scroll_chrome.by_owner.insert(owner, Vec::new());

        state.begin_epoch(session_identity, &next, &[]);

        assert!(state.scroll_chrome.by_owner.contains_key(&owner));
        let _ = successor.shutdown();
        let _ = session.shutdown();
    }
}
