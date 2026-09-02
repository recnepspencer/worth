use std::collections::BTreeSet;

use super::super::{
    UiOverlayBackdropInstanceScope, UiOverlayChangeSet, UiOverlayChangedBasis,
    UiOverlayOwnerSources, UiOverlayStackParticipant, UiOverlayStackSnapshot,
};
use super::owner_support::{
    idempotency, nested_open_request, open_request, owner_portal, owner_state,
};
use super::support::declaration;
use crate::runtime::allocation_receipt::{
    UiCommittedOverlayExtentBounds, UiMountedOverlayExtentOwner,
};
use crate::runtime::motion::UiMotionRuntimeState;
use crate::runtime::portal::{
    UiPortalDismissalPreparation, UiPortalDismissalTrigger, UiPortalIdentity,
    UiPortalLifecyclePosture, UiPortalOverlayBindingOwner, UiPortalRuntimeState,
    UiPortalServiceDisposition,
};
use crate::runtime::presentation_state::{
    UiApplicationPresentationOwnerExport, UiApplicationPresentationState,
};

pub(crate) struct OwnerIntegrationWorld {
    pub(crate) generation:
        crate::facade::prepared_application_authority::WorthUiPreparedApplicationGenerationIdentity,
    pub(crate) surface: worth_ui_dsl::UiSemanticSurfaceDeclarationIdentity,
    pub(crate) portal_declaration: worth_ui_dsl::UiPortalDeclarationId,
    pub(crate) backdrop_identity: worth_ui_dsl::UiBackdropIdentity,
    pub(crate) runtime_surface: worth_ui_host_contract::UiSemanticSurfaceIdentity,
    pub(crate) parent: UiPortalIdentity,
    pub(crate) child: UiPortalIdentity,
    pub(crate) sibling: UiPortalIdentity,
    pub(crate) portal_owner: UiPortalRuntimeState,
    pub(crate) extent: UiMountedOverlayExtentOwner,
    pub(crate) presentation: UiApplicationPresentationOwnerExport,
    pub(crate) bindings: UiPortalOverlayBindingOwner,
    pub(crate) motion: UiMotionRuntimeState,
    backdrop: Option<worth_ui_dsl::UiBackdropDeclaration>,
}

impl OwnerIntegrationWorld {
    pub(crate) fn new() -> Self {
        let app = prepared_app();
        let generation = app.generation_identity().clone();
        let surface = worth_ui_dsl::UiSemanticSurfaceDeclarationIdentity::new(3162).unwrap();
        let portal_declaration = worth_ui_dsl::UiPortalDeclarationId::new(31620).unwrap();
        let runtime_surface =
            worth_ui_host_contract::UiSemanticSurfaceIdentity::mint_unbound().unwrap();
        let (portal_owner, parent, child, sibling) = open_portals(runtime_surface);
        let bindings = bind_portals(
            &generation,
            runtime_surface,
            portal_declaration,
            [parent, child, sibling],
        );
        let extent = extent_owner(generation.clone(), surface, runtime_surface, 1);
        let presentation = presentation_export(&app, &generation);
        let motion =
            UiMotionRuntimeState::new(crate::runtime::UiServiceStatePersistencePosture::Ephemeral);
        let backdrop = backdrop_declaration(surface, portal_declaration);
        let backdrop_identity = backdrop.identity();
        Self {
            generation,
            surface,
            portal_declaration,
            backdrop_identity,
            runtime_surface,
            parent,
            child,
            sibling,
            portal_owner,
            extent,
            presentation,
            bindings,
            motion,
            backdrop: Some(backdrop),
        }
    }

    pub(crate) fn changes(&self) -> UiOverlayChangeSet {
        UiOverlayChangeSet::from_changes([UiOverlayChangedBasis::Portal(self.portal_declaration)])
    }

    pub(crate) fn sources(&self) -> UiOverlayOwnerSources<'_> {
        UiOverlayOwnerSources {
            generation: &self.generation,
            portal: &self.portal_owner,
            extent: &self.extent,
            presentation: &self.presentation,
            bindings: &self.bindings,
            motion: &self.motion,
        }
    }

    pub(crate) fn sources_with_extent<'a>(
        &'a self,
        extent: &'a UiMountedOverlayExtentOwner,
    ) -> UiOverlayOwnerSources<'a> {
        UiOverlayOwnerSources {
            generation: &self.generation,
            portal: &self.portal_owner,
            extent,
            presentation: &self.presentation,
            bindings: &self.bindings,
            motion: &self.motion,
        }
    }

    pub(crate) fn take_backdrop(&mut self) -> worth_ui_dsl::UiBackdropDeclaration {
        self.backdrop
            .take()
            .expect("owner integration owns one admitted backdrop")
    }
}

fn open_portals(
    runtime_surface: worth_ui_host_contract::UiSemanticSurfaceIdentity,
) -> (
    UiPortalRuntimeState,
    UiPortalIdentity,
    UiPortalIdentity,
    UiPortalIdentity,
) {
    let parent = owner_portal(31622);
    let child = owner_portal(31623);
    let sibling = owner_portal(31624);
    let mut owner = owner_state();
    commit_non_idempotent_open(&mut owner, open_request(parent, runtime_surface, 31622));
    commit_non_idempotent_open(
        &mut owner,
        nested_open_request(child, parent, runtime_surface, 31623),
    );
    commit_non_idempotent_open(&mut owner, open_request(sibling, runtime_surface, 31624));
    (owner, parent, child, sibling)
}

fn bind_portals(
    generation: &crate::facade::prepared_application_authority::
        WorthUiPreparedApplicationGenerationIdentity,
    runtime_surface: worth_ui_host_contract::UiSemanticSurfaceIdentity,
    portal_declaration: worth_ui_dsl::UiPortalDeclarationId,
    portals: [UiPortalIdentity; 3],
) -> UiPortalOverlayBindingOwner {
    let mut bindings = UiPortalOverlayBindingOwner::new(generation.clone(), runtime_surface);
    for portal in portals {
        bindings.bind(portal_declaration, portal).unwrap();
    }
    bindings
}

fn presentation_export(
    app: &crate::facade::entry::WorthUiHostNeutralApp,
    generation: &crate::facade::prepared_application_authority::
        WorthUiPreparedApplicationGenerationIdentity,
) -> UiApplicationPresentationOwnerExport {
    UiApplicationPresentationState::activate(app.capabilities()).overlay_owner_export(
        generation.clone(),
        worth_ui_host_contract::UiMountedPresentationAttemptIdentity::mint_unbound().unwrap(),
    )
}

fn backdrop_declaration(
    surface: worth_ui_dsl::UiSemanticSurfaceDeclarationIdentity,
    portal_declaration: worth_ui_dsl::UiPortalDeclarationId,
) -> worth_ui_dsl::UiBackdropDeclaration {
    declaration(
        31621,
        surface,
        worth_ui_dsl::UiBackdropScope::PerPortalInstance(portal_declaration),
        worth_ui_dsl::UiBackdropPresenceBasis::WhilePortalPresented(portal_declaration),
        worth_ui_dsl::UiBackdropMotionBasis::None,
        worth_ui_dsl::UiBackdropPlacement::ImmediatelyBeforePortal(portal_declaration),
    )
}

pub(crate) fn extent_owner(
    generation: crate::facade::prepared_application_authority::
        WorthUiPreparedApplicationGenerationIdentity,
    surface: worth_ui_dsl::UiSemanticSurfaceDeclarationIdentity,
    runtime_surface: worth_ui_host_contract::UiSemanticSurfaceIdentity,
    revision: u64,
) -> UiMountedOverlayExtentOwner {
    UiMountedOverlayExtentOwner::new(
        generation,
        surface,
        runtime_surface,
        revision,
        UiCommittedOverlayExtentBounds::new(0.0, 0.0, 800.0, 600.0).unwrap(),
        [],
    )
    .unwrap()
}

pub(crate) fn prepared_app() -> crate::facade::entry::WorthUiHostNeutralApp {
    crate::facade::WorthUi::app()
        .with_change_profile(crate::runtime::rebind::UiChangeProfile::platform_pulse())
        .freeze()
        .unwrap()
}

pub(crate) fn commit_non_idempotent_open(
    owner: &mut UiPortalRuntimeState,
    request: crate::runtime::portal::UiPortalServiceRequest,
) {
    let transition = owner.prepare(request).unwrap();
    assert!(!transition.is_idempotent());
    let receipt = owner.commit_published(transition).unwrap();
    assert_eq!(receipt.disposition(), UiPortalServiceDisposition::Opened);
}

pub(crate) fn close_topmost(owner: &mut UiPortalRuntimeState, lineage: u64) -> UiPortalIdentity {
    let dismissal = owner
        .prepare_dismissal(UiPortalDismissalTrigger::Escape, None, idempotency(lineage))
        .unwrap();
    let transition = match dismissal {
        UiPortalDismissalPreparation::Prepared(prepared) => prepared.into_transition(),
        UiPortalDismissalPreparation::Ignored(reason) => {
            panic!("topmost Portal dismissal was ignored: {reason:?}")
        }
    };
    let portal = transition.portal();
    owner.commit_published(transition).unwrap();
    portal
}

fn portal_facts(
    snapshot: &UiOverlayStackSnapshot,
) -> Vec<(
    UiPortalIdentity,
    Option<UiPortalIdentity>,
    u64,
    UiPortalLifecyclePosture,
)> {
    snapshot
        .participants()
        .iter()
        .filter_map(|participant| match participant {
            UiOverlayStackParticipant::Portal(row) => Some((
                row.portal(),
                row.parent(),
                row.ordinal().value(),
                row.lifecycle(),
            )),
            UiOverlayStackParticipant::Backdrop(_) => None,
        })
        .collect()
}

fn backdrop_facts(
    snapshot: &UiOverlayStackSnapshot,
) -> Vec<(
    worth_ui_dsl::UiBackdropIdentity,
    UiOverlayBackdropInstanceScope,
)> {
    snapshot
        .participants()
        .iter()
        .filter_map(|participant| match participant {
            UiOverlayStackParticipant::Portal(_) => None,
            UiOverlayStackParticipant::Backdrop(row) => {
                Some((row.declaration(), row.identity().scope()))
            }
        })
        .collect()
}

pub(crate) fn assert_overlay(
    snapshot: &UiOverlayStackSnapshot,
    backdrop: worth_ui_dsl::UiBackdropIdentity,
    expected_portals: &[(
        UiPortalIdentity,
        Option<UiPortalIdentity>,
        u64,
        UiPortalLifecyclePosture,
    )],
) {
    assert_eq!(portal_facts(snapshot), expected_portals);
    let backdrops = backdrop_facts(snapshot);
    assert_eq!(backdrops.len(), expected_portals.len());
    assert_eq!(
        backdrops
            .iter()
            .filter(|(declaration, _)| *declaration == backdrop)
            .count(),
        expected_portals.len()
    );
    let identities = backdrops.iter().copied().collect::<BTreeSet<_>>();
    assert_eq!(identities.len(), expected_portals.len());
    for (declaration, scope) in backdrops {
        assert_eq!(declaration, backdrop);
        assert!(matches!(scope, UiOverlayBackdropInstanceScope::Portal(_)));
    }
    let mut participants = snapshot.participants().iter();
    for (portal, _, _, _) in expected_portals {
        let Some(UiOverlayStackParticipant::Backdrop(row)) = participants.next() else {
            panic!("expected a backdrop before Portal {portal:?}")
        };
        assert_eq!(row.declaration(), backdrop);
        assert_eq!(
            row.identity().scope(),
            UiOverlayBackdropInstanceScope::Portal(*portal)
        );
        let Some(UiOverlayStackParticipant::Portal(row)) = participants.next() else {
            panic!("expected Portal {portal:?} after its backdrop")
        };
        assert_eq!(row.portal(), *portal);
    }
    assert!(participants.next().is_none());
    assert_eq!(
        snapshot.participants().len(),
        expected_portals.len().saturating_mul(2)
    );
}
