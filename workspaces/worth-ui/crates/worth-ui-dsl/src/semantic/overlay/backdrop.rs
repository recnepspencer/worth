#[derive(Clone, Debug, Eq, PartialEq)]
pub struct UiBackdropDeclaration {
    identity: super::UiBackdropIdentity,
    surface: super::UiSemanticSurfaceDeclarationIdentity,
    scope: super::UiBackdropScope,
    extent: super::UiBackdropExtentBasis,
    presence: super::UiBackdropPresenceBasis,
    motion: super::UiBackdropMotionBasis,
    placement: super::UiBackdropPlacement,
    role: super::super::UiAppearanceRoleIdentity,
    role_revision: super::super::UiAppearanceRoleRevision,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum UiBackdropDeclarationDenial {
    PerPortalScopeMismatch,
    ForeignSurfaceExtent,
    ForeignPortalPlacement,
    IncompatibleAppearanceRole,
}

impl UiBackdropDeclaration {
    pub fn admit(
        identity: super::UiBackdropIdentity,
        surface: super::UiSemanticSurfaceDeclarationIdentity,
        scope: super::UiBackdropScope,
        extent: super::UiBackdropExtentBasis,
        presence: super::UiBackdropPresenceBasis,
        motion: super::UiBackdropMotionBasis,
        placement: super::UiBackdropPlacement,
        role: &super::super::UiAppearanceRoleDeclaration,
    ) -> Result<Self, UiBackdropDeclarationDenial> {
        Self::admit_with_role_revision(
            identity,
            surface,
            scope,
            extent,
            presence,
            motion,
            placement,
            role.revision(),
            role,
        )
    }

    pub fn admit_with_role_revision(
        identity: super::UiBackdropIdentity,
        surface: super::UiSemanticSurfaceDeclarationIdentity,
        scope: super::UiBackdropScope,
        extent: super::UiBackdropExtentBasis,
        presence: super::UiBackdropPresenceBasis,
        motion: super::UiBackdropMotionBasis,
        placement: super::UiBackdropPlacement,
        role_revision: super::super::UiAppearanceRoleRevision,
        role: &super::super::UiAppearanceRoleDeclaration,
    ) -> Result<Self, UiBackdropDeclarationDenial> {
        if extent.surface() != surface {
            return Err(UiBackdropDeclarationDenial::ForeignSurfaceExtent);
        }
        if let super::UiBackdropScope::PerPortalInstance(portal) = scope {
            let matches = matches!(presence, super::UiBackdropPresenceBasis::WhilePortalPresented(value) if value == portal)
                && !matches!(motion, super::UiBackdropMotionBasis::PortalPresentation(value) if value != portal);
            if !matches {
                return Err(UiBackdropDeclarationDenial::PerPortalScopeMismatch);
            }
            if placement
                .portal_anchor()
                .is_some_and(|anchor| anchor != portal)
            {
                return Err(UiBackdropDeclarationDenial::ForeignPortalPlacement);
            }
        }
        if role.aspect_contract() != &super::super::UiAppearanceAspectContract::backdrop()
            || role
                .partitions()
                .iter()
                .any(|(_, partition)| !partition.axes().is_empty())
        {
            return Err(UiBackdropDeclarationDenial::IncompatibleAppearanceRole);
        }
        Ok(Self {
            identity,
            surface,
            scope,
            extent,
            presence,
            motion,
            placement,
            role: role.role().clone(),
            role_revision,
        })
    }

    pub const fn identity(&self) -> super::UiBackdropIdentity {
        self.identity
    }
    pub const fn surface(&self) -> super::UiSemanticSurfaceDeclarationIdentity {
        self.surface
    }
    pub const fn scope(&self) -> super::UiBackdropScope {
        self.scope
    }
    pub const fn extent(&self) -> super::UiBackdropExtentBasis {
        self.extent
    }
    pub const fn presence(&self) -> super::UiBackdropPresenceBasis {
        self.presence
    }
    pub const fn motion(&self) -> super::UiBackdropMotionBasis {
        self.motion
    }
    pub const fn placement(&self) -> super::UiBackdropPlacement {
        self.placement
    }
    pub const fn role(&self) -> &super::super::UiAppearanceRoleIdentity {
        &self.role
    }
    pub const fn role_revision(&self) -> super::super::UiAppearanceRoleRevision {
        self.role_revision
    }

    pub fn canonical_bytes(&self) -> Vec<u8> {
        let mut bytes = b"worth-ui:backdrop:v3".to_vec();
        bytes.extend_from_slice(&self.identity.value().to_le_bytes());
        bytes.extend_from_slice(&self.surface.value().to_le_bytes());
        encode_scope(&mut bytes, self.scope);
        encode_extent(&mut bytes, self.extent);
        encode_presence(&mut bytes, self.presence);
        encode_motion(&mut bytes, self.motion);
        encode_placement(&mut bytes, self.placement);
        text(&mut bytes, self.role.as_str());
        bytes.extend_from_slice(&self.role_revision.value().to_le_bytes());
        bytes
    }
}

fn encode_scope(bytes: &mut Vec<u8>, scope: super::UiBackdropScope) {
    match scope {
        super::UiBackdropScope::SurfaceSingleton => bytes.push(1),
        super::UiBackdropScope::PerPortalInstance(portal) => {
            bytes.push(2);
            bytes.extend_from_slice(&portal.value().to_le_bytes());
        }
    }
}

fn encode_extent(bytes: &mut Vec<u8>, extent: super::UiBackdropExtentBasis) {
    match extent {
        super::UiBackdropExtentBasis::SurfaceViewport(surface) => {
            bytes.push(1);
            bytes.extend_from_slice(&surface.value().to_le_bytes());
        }
        super::UiBackdropExtentBasis::PresentedMosaicRegion { surface, region } => {
            bytes.push(2);
            bytes.extend_from_slice(&surface.value().to_le_bytes());
            bytes.extend_from_slice(&region.value().to_le_bytes());
        }
    }
}

fn encode_presence(bytes: &mut Vec<u8>, presence: super::UiBackdropPresenceBasis) {
    match presence {
        super::UiBackdropPresenceBasis::Always => bytes.push(1),
        super::UiBackdropPresenceBasis::WhilePortalPresented(portal) => {
            bytes.push(2);
            bytes.extend_from_slice(&portal.value().to_le_bytes());
        }
    }
}

fn encode_motion(bytes: &mut Vec<u8>, motion: super::UiBackdropMotionBasis) {
    match motion {
        super::UiBackdropMotionBasis::None => bytes.push(1),
        super::UiBackdropMotionBasis::PortalPresentation(portal) => {
            bytes.push(2);
            bytes.extend_from_slice(&portal.value().to_le_bytes());
        }
    }
}

fn encode_placement(bytes: &mut Vec<u8>, placement: super::UiBackdropPlacement) {
    match placement {
        super::UiBackdropPlacement::AboveSurfaceContent => bytes.push(1),
        super::UiBackdropPlacement::ImmediatelyBeforePortal(portal) => {
            bytes.push(2);
            bytes.extend_from_slice(&portal.value().to_le_bytes());
        }
        super::UiBackdropPlacement::ImmediatelyAfterPortal(portal) => {
            bytes.push(3);
            bytes.extend_from_slice(&portal.value().to_le_bytes());
        }
        super::UiBackdropPlacement::ImmediatelyBeforeBackdrop(backdrop) => {
            bytes.push(4);
            bytes.extend_from_slice(&backdrop.value().to_le_bytes());
        }
        super::UiBackdropPlacement::ImmediatelyAfterBackdrop(backdrop) => {
            bytes.push(5);
            bytes.extend_from_slice(&backdrop.value().to_le_bytes());
        }
    }
}

fn text(bytes: &mut Vec<u8>, value: &str) {
    bytes.extend_from_slice(&(value.len() as u64).to_le_bytes());
    bytes.extend_from_slice(value.as_bytes());
}

#[cfg(test)]
#[path = "backdrop_tests.rs"]
mod tests;
