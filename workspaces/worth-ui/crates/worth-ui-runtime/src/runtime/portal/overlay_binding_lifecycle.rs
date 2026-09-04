use std::collections::BTreeMap;

use crate::runtime::intent::WorthUiActiveApplicationGenerationIdentity;
use crate::runtime::source_ingress::WorthUiAuthoredOverlayMaterial;
use worth_ui_dsl::{UiPortalDeclarationId, UiSemanticSurfaceDeclarationIdentity};
use worth_ui_host_contract::UiSemanticSurfaceIdentity;

use super::{
    UiPortalIdentity, UiPortalLifecyclePosture, UiPortalOverlayBindingDenial,
    UiPortalOverlayBindingOwner, UiPortalOverlayBindingOwnerExport, UiPortalRuntimeState,
    UiPreparedPortalServiceTransition,
};

const DECLARED_SURFACE_BINDING_LIMIT: usize = 256;

/// Denials produced by the live declaration-to-mounted Portal binding owner.
/// None of these paths mint or reinterpret a compiler identity.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum UiPortalOverlayBindingLifecycleDenial {
    ForeignGeneration,
    UnknownPortalDeclaration,
    ForeignSurfaceDeclaration,
    PortalSurfaceUndeclared,
    PortalSurfaceMismatch,
    DeclaredSurfaceUnbound,
    DeclaredSurfaceAlreadyBound,
    RuntimeSurfaceConflict,
    Mounted(crate::mounting::UiMountedIdentityDenial),
    PortalDeclarationConflict,
    PortalAlreadyLiveWithoutBinding,
    RetiredBinding,
    TransitionMismatch,
    SurfaceBindingCapacityExceeded,
    Owner(UiPortalOverlayBindingDenial),
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct UiPortalOverlayBindingStage {
    generation: WorthUiActiveApplicationGenerationIdentity,
    declaration: UiPortalDeclarationId,
    surface_declaration: UiSemanticSurfaceDeclarationIdentity,
    runtime_surface: UiSemanticSurfaceIdentity,
    portal: UiPortalIdentity,
    already_bound: bool,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct UiPortalOverlayBindingCommit {
    stage: Option<UiPortalOverlayBindingStage>,
    portal: UiPortalIdentity,
    runtime_surface: UiSemanticSurfaceIdentity,
    declared_portal: Option<UiPortalDeclarationId>,
    opens: bool,
    idempotent: bool,
    posture: UiPortalLifecyclePosture,
    closed_descendants: Box<[UiPortalIdentity]>,
}

pub(crate) struct UiPortalOverlayBindingLifecycle {
    generation: WorthUiActiveApplicationGenerationIdentity,
    surface_bindings: BTreeMap<UiSemanticSurfaceDeclarationIdentity, UiSemanticSurfaceIdentity>,
    owners: BTreeMap<UiSemanticSurfaceIdentity, UiPortalOverlayBindingOwner>,
}

impl UiPortalOverlayBindingLifecycle {
    pub(crate) fn new(generation: WorthUiActiveApplicationGenerationIdentity) -> Self {
        Self {
            generation,
            surface_bindings: BTreeMap::new(),
            owners: BTreeMap::new(),
        }
    }

    pub(crate) fn validate_declared_surface(
        &self,
        generation: &WorthUiActiveApplicationGenerationIdentity,
        material: &WorthUiAuthoredOverlayMaterial,
        declaration: UiSemanticSurfaceDeclarationIdentity,
    ) -> Result<(), UiPortalOverlayBindingLifecycleDenial> {
        self.require_generation(generation)?;
        if !material
            .overlay_declaration_bindings()
            .contains_surface(declaration)
        {
            return Err(UiPortalOverlayBindingLifecycleDenial::ForeignSurfaceDeclaration);
        }
        if self.surface_bindings.contains_key(&declaration) {
            return Err(UiPortalOverlayBindingLifecycleDenial::DeclaredSurfaceAlreadyBound);
        }
        if self.surface_bindings.len() >= DECLARED_SURFACE_BINDING_LIMIT {
            return Err(UiPortalOverlayBindingLifecycleDenial::SurfaceBindingCapacityExceeded);
        }
        Ok(())
    }

    pub(crate) fn install_declared_surface(
        &mut self,
        generation: &WorthUiActiveApplicationGenerationIdentity,
        declaration: UiSemanticSurfaceDeclarationIdentity,
        runtime_surface: UiSemanticSurfaceIdentity,
    ) -> Result<(), UiPortalOverlayBindingLifecycleDenial> {
        self.require_generation(generation)?;
        if self.surface_bindings.contains_key(&declaration) {
            return Err(UiPortalOverlayBindingLifecycleDenial::DeclaredSurfaceAlreadyBound);
        }
        if self.surface_bindings.len() >= DECLARED_SURFACE_BINDING_LIMIT {
            return Err(UiPortalOverlayBindingLifecycleDenial::SurfaceBindingCapacityExceeded);
        }
        if self
            .surface_bindings
            .values()
            .any(|candidate| *candidate == runtime_surface)
        {
            return Err(UiPortalOverlayBindingLifecycleDenial::RuntimeSurfaceConflict);
        }
        self.surface_bindings.insert(declaration, runtime_surface);
        self.owners.insert(
            runtime_surface,
            UiPortalOverlayBindingOwner::new(
                self.generation.prepared_generation().clone(),
                runtime_surface,
            ),
        );
        Ok(())
    }

    pub(crate) fn admit_open(
        &self,
        generation: &WorthUiActiveApplicationGenerationIdentity,
        material: &WorthUiAuthoredOverlayMaterial,
        mounted: &crate::mounting::WorthUiMountedSessionState,
        portal_state: &UiPortalRuntimeState,
        declaration: UiPortalDeclarationId,
        portal: UiPortalIdentity,
        runtime_surface: UiSemanticSurfaceIdentity,
    ) -> Result<UiPortalOverlayBindingStage, UiPortalOverlayBindingLifecycleDenial> {
        self.require_generation(generation)?;
        let mounted_basis = mounted
            .current_mounted_identity_basis(portal.owner().mounted_instance_identity())
            .ok_or(UiPortalOverlayBindingLifecycleDenial::Mounted(
                crate::mounting::UiMountedIdentityDenial::UnknownMountedInstance,
            ))?;
        if mounted_basis.graph_node_identity() != portal.owner().graph_node() {
            return Err(UiPortalOverlayBindingLifecycleDenial::Mounted(
                crate::mounting::UiMountedIdentityDenial::ForeignGraphWorld,
            ));
        }
        if mounted_basis.semantic_surface_identity() != runtime_surface {
            return Err(UiPortalOverlayBindingLifecycleDenial::PortalSurfaceMismatch);
        }
        let binding = material
            .portal_anchor_binding(declaration)
            .ok_or(UiPortalOverlayBindingLifecycleDenial::UnknownPortalDeclaration)?;
        let surface_declaration = binding
            .surface_declaration_id()
            .ok_or(UiPortalOverlayBindingLifecycleDenial::PortalSurfaceUndeclared)?;
        let Some(bound_surface) = self.surface_bindings.get(&surface_declaration) else {
            return Err(UiPortalOverlayBindingLifecycleDenial::DeclaredSurfaceUnbound);
        };
        if *bound_surface != runtime_surface {
            return Err(UiPortalOverlayBindingLifecycleDenial::PortalSurfaceMismatch);
        }
        let Some(owner) = self.owners.get(bound_surface) else {
            return Err(UiPortalOverlayBindingLifecycleDenial::DeclaredSurfaceUnbound);
        };
        let existing = owner.binding_for_portal(portal);
        if let Some(current_surface) = portal_state.semantic_surface_for(portal) {
            if current_surface != runtime_surface {
                return Err(UiPortalOverlayBindingLifecycleDenial::PortalSurfaceMismatch);
            }
            match existing {
                Some(current) if current != declaration => {
                    return Err(UiPortalOverlayBindingLifecycleDenial::PortalDeclarationConflict)
                }
                Some(_) => {}
                None => {
                    return Err(
                        UiPortalOverlayBindingLifecycleDenial::PortalAlreadyLiveWithoutBinding,
                    )
                }
            }
        } else if existing.is_some() {
            return Err(UiPortalOverlayBindingLifecycleDenial::RetiredBinding);
        }
        Ok(UiPortalOverlayBindingStage {
            generation: generation.clone(),
            declaration,
            surface_declaration,
            runtime_surface,
            portal,
            already_bound: existing.is_some(),
        })
    }

    pub(crate) fn commit_published(
        &mut self,
        generation: &WorthUiActiveApplicationGenerationIdentity,
        commit: UiPortalOverlayBindingCommit,
    ) -> Result<(), UiPortalOverlayBindingLifecycleDenial> {
        self.require_generation(generation)?;
        if let Some(stage) = commit.stage.as_ref() {
            if stage.generation() != &self.generation
                || !commit.opens
                || commit.declared_portal != Some(stage.declaration())
                || commit.portal != stage.portal
                || commit.runtime_surface != stage.runtime_surface
                || self.surface_bindings.get(&stage.surface_declaration)
                    != Some(&stage.runtime_surface)
            {
                return Err(UiPortalOverlayBindingLifecycleDenial::TransitionMismatch);
            }
            let owner = self
                .owners
                .get_mut(&stage.runtime_surface)
                .ok_or(UiPortalOverlayBindingLifecycleDenial::DeclaredSurfaceUnbound)?;
            if !commit.idempotent && !stage.already_bound {
                owner
                    .bind(stage.declaration, stage.portal)
                    .map_err(UiPortalOverlayBindingLifecycleDenial::Owner)?;
            } else if owner.binding_for_portal(stage.portal) != Some(stage.declaration) {
                return Err(UiPortalOverlayBindingLifecycleDenial::RetiredBinding);
            }
        }
        if !commit.opens {
            for descendant in commit.closed_descendants.iter().copied() {
                self.retire_portal(descendant);
            }
            if commit.posture == UiPortalLifecyclePosture::Closed {
                self.retire_portal(commit.portal);
            }
        }
        Ok(())
    }

    pub(crate) fn exports(
        &self,
        generation: &WorthUiActiveApplicationGenerationIdentity,
        portal_state: Option<&UiPortalRuntimeState>,
    ) -> Result<Box<[UiPortalOverlayBindingOwnerExport]>, UiPortalOverlayBindingLifecycleDenial>
    {
        self.require_generation(generation)?;
        let Some(portal_state) = portal_state else {
            return Ok(Box::new([]));
        };
        let snapshot = portal_state.stack_snapshot();
        self.owners
            .values()
            .filter(|owner| !owner.is_empty())
            .map(|owner| {
                owner
                    .export(&snapshot)
                    .map_err(UiPortalOverlayBindingLifecycleDenial::Owner)
            })
            .collect()
    }

    pub(crate) fn replace_generation(
        &mut self,
        generation: WorthUiActiveApplicationGenerationIdentity,
    ) {
        self.generation = generation;
        self.surface_bindings.clear();
        self.owners.clear();
    }

    pub(crate) fn clear_for_shutdown(&mut self) {
        self.surface_bindings.clear();
        self.owners.clear();
    }

    fn require_generation(
        &self,
        generation: &WorthUiActiveApplicationGenerationIdentity,
    ) -> Result<(), UiPortalOverlayBindingLifecycleDenial> {
        (self.generation == *generation)
            .then_some(())
            .ok_or(UiPortalOverlayBindingLifecycleDenial::ForeignGeneration)
    }

    fn retire_portal(&mut self, portal: UiPortalIdentity) {
        for owner in self.owners.values_mut() {
            owner.remove(portal);
        }
    }
}

impl UiPortalOverlayBindingStage {
    pub(crate) fn generation(&self) -> &WorthUiActiveApplicationGenerationIdentity {
        &self.generation
    }

    pub(crate) const fn declaration(&self) -> UiPortalDeclarationId {
        self.declaration
    }
}

impl UiPortalOverlayBindingCommit {
    pub(crate) fn from_transition(
        transition: &UiPreparedPortalServiceTransition,
        stage: Option<UiPortalOverlayBindingStage>,
    ) -> Self {
        let request = transition.request();
        Self {
            stage,
            portal: transition.portal(),
            runtime_surface: request.semantic_surface(),
            declared_portal: request.declared_portal(),
            opens: transition.opens_portal(),
            idempotent: transition.is_idempotent(),
            posture: transition.staged_posture(),
            closed_descendants: transition.closed_descendants().to_vec().into_boxed_slice(),
        }
    }

    pub(crate) fn with_retained_exit(mut self, retained: bool) -> Self {
        if !self.opens {
            self.posture = if retained {
                UiPortalLifecyclePosture::Closing
            } else {
                UiPortalLifecyclePosture::Closed
            };
        }
        self
    }
}

impl UiPortalOverlayBindingLifecycleDenial {
    pub(crate) const fn stop_reason(
        self,
    ) -> crate::runtime::intent_execution::UiIntentPortalBindingStopReason {
        use crate::runtime::intent_execution::UiIntentPortalBindingStopReason as Stop;
        match self {
            Self::ForeignGeneration => Stop::ForeignGeneration,
            Self::UnknownPortalDeclaration => Stop::UnknownPortalDeclaration,
            Self::ForeignSurfaceDeclaration => Stop::ForeignSurfaceDeclaration,
            Self::PortalSurfaceUndeclared => Stop::PortalSurfaceUndeclared,
            Self::DeclaredSurfaceUnbound => Stop::DeclaredSurfaceUnbound,
            Self::DeclaredSurfaceAlreadyBound => Stop::DeclaredSurfaceAlreadyBound,
            Self::RuntimeSurfaceConflict => Stop::RuntimeSurfaceConflict,
            Self::PortalSurfaceMismatch => Stop::PortalSurfaceMismatch,
            Self::PortalDeclarationConflict => Stop::PortalDeclarationConflict,
            Self::PortalAlreadyLiveWithoutBinding => Stop::PortalAlreadyLiveWithoutBinding,
            Self::RetiredBinding => Stop::RetiredBinding,
            Self::TransitionMismatch => Stop::TransitionMismatch,
            Self::SurfaceBindingCapacityExceeded => Stop::SurfaceBindingCapacityExceeded,
            Self::Owner(_) => Stop::OwnerConflict,
            Self::Mounted(_) => Stop::MountedIdentity,
        }
    }
}
