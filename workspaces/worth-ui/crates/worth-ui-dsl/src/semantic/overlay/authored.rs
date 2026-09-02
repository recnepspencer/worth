use crate::{UiAppearanceRoleIdentity, UiAppearanceRoleRevision};

#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum UiStaticBackdropScope {
    SurfaceSingleton,
    PerPortalInstance(Box<str>),
}

#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum UiStaticBackdropExtent {
    SurfaceViewport(Box<str>),
    PresentedMosaicRegion { surface: Box<str>, region: Box<str> },
}

#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum UiStaticBackdropPresence {
    Always,
    WhilePortalPresented(Box<str>),
}

#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum UiStaticBackdropMotion {
    None,
    PortalPresentation(Box<str>),
}

#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum UiStaticBackdropPlacement {
    AboveSurfaceContent,
    ImmediatelyBeforePortal(Box<str>),
    ImmediatelyAfterPortal(Box<str>),
    ImmediatelyBeforeBackdrop(Box<str>),
    ImmediatelyAfterBackdrop(Box<str>),
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct UiStaticBackdropDeclaration {
    identity: Box<str>,
    surface: Box<str>,
    scope: UiStaticBackdropScope,
    extent: UiStaticBackdropExtent,
    presence: UiStaticBackdropPresence,
    motion: UiStaticBackdropMotion,
    placement: UiStaticBackdropPlacement,
    role: UiAppearanceRoleIdentity,
    role_revision: UiAppearanceRoleRevision,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct UiBackdropDeclarationAuthoring {
    source: UiBackdropDeclarationSource,
}

#[derive(Clone, Debug, Eq, PartialEq)]
struct UiBackdropDeclarationSource {
    identity: Box<str>,
    surface: Box<str>,
    scope: Option<UiStaticBackdropScope>,
    extent: Option<UiStaticBackdropExtent>,
    presence: Option<UiStaticBackdropPresence>,
    motion: Option<UiStaticBackdropMotion>,
    placement: Option<UiStaticBackdropPlacement>,
    role: UiAppearanceRoleIdentity,
    role_revision: UiAppearanceRoleRevision,
    duplicate_clause: bool,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum UiBackdropDeclarationAuthoringDenial {
    EmptyIdentity,
    EmptySurface,
    MissingScope,
    MissingExtent,
    MissingPresence,
    MissingPlacement,
    DuplicateClause,
    ForeignSurfaceExtent,
    PerPortalScopeMismatch,
    ForeignPortalPlacement,
}

impl UiBackdropDeclarationAuthoring {
    pub fn new(
        identity: impl Into<Box<str>>,
        surface: impl Into<Box<str>>,
        role: UiAppearanceRoleIdentity,
        role_revision: UiAppearanceRoleRevision,
    ) -> Result<Self, UiBackdropDeclarationAuthoringDenial> {
        let identity = identity.into();
        let surface = surface.into();
        if identity.is_empty() {
            return Err(UiBackdropDeclarationAuthoringDenial::EmptyIdentity);
        }
        if surface.is_empty() {
            return Err(UiBackdropDeclarationAuthoringDenial::EmptySurface);
        }
        Ok(Self {
            source: UiBackdropDeclarationSource {
                identity,
                surface,
                scope: None,
                extent: None,
                presence: None,
                motion: None,
                placement: None,
                role,
                role_revision,
                duplicate_clause: false,
            },
        })
    }

    pub fn with_scope(mut self, scope: UiStaticBackdropScope) -> Self {
        if self.source.scope.replace(scope).is_some() {
            self.source.duplicate_clause = true;
        }
        self
    }

    pub fn with_extent(mut self, extent: UiStaticBackdropExtent) -> Self {
        if self.source.extent.replace(extent).is_some() {
            self.source.duplicate_clause = true;
        }
        self
    }

    pub fn with_presence(mut self, presence: UiStaticBackdropPresence) -> Self {
        if self.source.presence.replace(presence).is_some() {
            self.source.duplicate_clause = true;
        }
        self
    }

    pub fn with_motion(mut self, motion: UiStaticBackdropMotion) -> Self {
        if self.source.motion.replace(motion).is_some() {
            self.source.duplicate_clause = true;
        }
        self
    }

    pub fn with_placement(mut self, placement: UiStaticBackdropPlacement) -> Self {
        if self.source.placement.replace(placement).is_some() {
            self.source.duplicate_clause = true;
        }
        self
    }

    pub fn admit(
        self,
    ) -> Result<UiStaticBackdropDeclaration, UiBackdropDeclarationAuthoringDenial> {
        let UiBackdropDeclarationSource {
            identity,
            surface,
            scope,
            extent,
            presence,
            motion,
            placement,
            role,
            role_revision,
            duplicate_clause,
        } = self.source;
        if duplicate_clause {
            return Err(UiBackdropDeclarationAuthoringDenial::DuplicateClause);
        }
        let scope = scope.ok_or(UiBackdropDeclarationAuthoringDenial::MissingScope)?;
        let extent = extent.ok_or(UiBackdropDeclarationAuthoringDenial::MissingExtent)?;
        let presence = presence.ok_or(UiBackdropDeclarationAuthoringDenial::MissingPresence)?;
        let motion = motion.unwrap_or(UiStaticBackdropMotion::None);
        let placement = placement.ok_or(UiBackdropDeclarationAuthoringDenial::MissingPlacement)?;
        if extent.surface() != surface.as_ref() {
            return Err(UiBackdropDeclarationAuthoringDenial::ForeignSurfaceExtent);
        }
        if let UiStaticBackdropScope::PerPortalInstance(portal) = &scope {
            if !matches!(&presence, UiStaticBackdropPresence::WhilePortalPresented(value) if value == portal)
                || matches!(&motion, UiStaticBackdropMotion::PortalPresentation(value) if value != portal)
            {
                return Err(UiBackdropDeclarationAuthoringDenial::PerPortalScopeMismatch);
            }
            if placement
                .portal_anchor()
                .is_some_and(|anchor| anchor != portal.as_ref())
            {
                return Err(UiBackdropDeclarationAuthoringDenial::ForeignPortalPlacement);
            }
        }
        Ok(UiStaticBackdropDeclaration {
            identity,
            surface,
            scope,
            extent,
            presence,
            motion,
            placement,
            role,
            role_revision,
        })
    }
}

impl UiStaticBackdropDeclaration {
    pub fn identity(&self) -> &str {
        &self.identity
    }

    pub fn surface(&self) -> &str {
        &self.surface
    }

    pub fn scope(&self) -> &UiStaticBackdropScope {
        &self.scope
    }

    pub fn extent(&self) -> &UiStaticBackdropExtent {
        &self.extent
    }

    pub fn presence(&self) -> &UiStaticBackdropPresence {
        &self.presence
    }

    pub fn motion(&self) -> &UiStaticBackdropMotion {
        &self.motion
    }

    pub fn placement(&self) -> &UiStaticBackdropPlacement {
        &self.placement
    }

    pub fn role(&self) -> &UiAppearanceRoleIdentity {
        &self.role
    }

    pub const fn role_revision(&self) -> UiAppearanceRoleRevision {
        self.role_revision
    }

    pub fn canonical_bytes(&self) -> Vec<u8> {
        let mut bytes = b"worth-ui:backdrop:v2".to_vec();
        text(&mut bytes, &self.identity);
        text(&mut bytes, &self.surface);
        encode_scope(&mut bytes, &self.scope);
        encode_extent(&mut bytes, &self.extent);
        encode_presence(&mut bytes, &self.presence);
        encode_motion(&mut bytes, &self.motion);
        encode_placement(&mut bytes, &self.placement);
        text(&mut bytes, self.role.as_str());
        bytes.extend_from_slice(&self.role_revision.value().to_le_bytes());
        bytes
    }

    pub(crate) fn canonical_text(&self) -> String {
        self.canonical_bytes()
            .iter()
            .map(|byte| format!("{byte:02x}"))
            .collect()
    }
}

impl UiStaticBackdropExtent {
    pub fn surface(&self) -> &str {
        match self {
            Self::SurfaceViewport(surface) | Self::PresentedMosaicRegion { surface, .. } => surface,
        }
    }
}

impl UiStaticBackdropPlacement {
    pub fn portal_anchor(&self) -> Option<&str> {
        match self {
            Self::ImmediatelyBeforePortal(portal) | Self::ImmediatelyAfterPortal(portal) => {
                Some(portal)
            }
            _ => None,
        }
    }
}

fn encode_scope(bytes: &mut Vec<u8>, value: &UiStaticBackdropScope) {
    match value {
        UiStaticBackdropScope::SurfaceSingleton => bytes.push(1),
        UiStaticBackdropScope::PerPortalInstance(portal) => {
            bytes.push(2);
            text(bytes, portal);
        }
    }
}

fn encode_extent(bytes: &mut Vec<u8>, value: &UiStaticBackdropExtent) {
    match value {
        UiStaticBackdropExtent::SurfaceViewport(surface) => {
            bytes.push(1);
            text(bytes, surface);
        }
        UiStaticBackdropExtent::PresentedMosaicRegion { surface, region } => {
            bytes.push(2);
            text(bytes, surface);
            text(bytes, region);
        }
    }
}

fn encode_presence(bytes: &mut Vec<u8>, value: &UiStaticBackdropPresence) {
    match value {
        UiStaticBackdropPresence::Always => bytes.push(1),
        UiStaticBackdropPresence::WhilePortalPresented(portal) => {
            bytes.push(2);
            text(bytes, portal);
        }
    }
}

fn encode_motion(bytes: &mut Vec<u8>, value: &UiStaticBackdropMotion) {
    match value {
        UiStaticBackdropMotion::None => bytes.push(1),
        UiStaticBackdropMotion::PortalPresentation(portal) => {
            bytes.push(2);
            text(bytes, portal);
        }
    }
}

fn encode_placement(bytes: &mut Vec<u8>, value: &UiStaticBackdropPlacement) {
    match value {
        UiStaticBackdropPlacement::AboveSurfaceContent => bytes.push(1),
        UiStaticBackdropPlacement::ImmediatelyBeforePortal(anchor) => {
            bytes.push(2);
            text(bytes, anchor);
        }
        UiStaticBackdropPlacement::ImmediatelyAfterPortal(anchor) => {
            bytes.push(3);
            text(bytes, anchor);
        }
        UiStaticBackdropPlacement::ImmediatelyBeforeBackdrop(anchor) => {
            bytes.push(4);
            text(bytes, anchor);
        }
        UiStaticBackdropPlacement::ImmediatelyAfterBackdrop(anchor) => {
            bytes.push(5);
            text(bytes, anchor);
        }
    }
}

fn text(bytes: &mut Vec<u8>, value: &str) {
    bytes.extend_from_slice(&(value.len() as u64).to_le_bytes());
    bytes.extend_from_slice(value.as_bytes());
}

#[cfg(test)]
mod tests;
