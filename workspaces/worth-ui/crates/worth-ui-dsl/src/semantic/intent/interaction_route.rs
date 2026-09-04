use crate::source::WorthUiArtifactInputBodyAtom;

use super::WorthUiIntentInteractionFamily;

#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum WorthUiIntentInteractionRouteKind {
    Product,
    Confirmation,
}

#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub struct WorthUiIntentInteractionRoute {
    family: WorthUiIntentInteractionFamily,
    declaration_identity: Box<str>,
    kind: WorthUiIntentInteractionRouteKind,
    opened_portal_identity: Option<Box<str>>,
}

impl WorthUiIntentInteractionRoute {
    pub fn product(
        family: WorthUiIntentInteractionFamily,
        declaration_identity: impl Into<Box<str>>,
    ) -> Self {
        Self::new(
            family,
            declaration_identity,
            WorthUiIntentInteractionRouteKind::Product,
            None,
        )
    }

    pub fn confirmation(declaration_identity: impl Into<Box<str>>) -> Self {
        Self::new(
            WorthUiIntentInteractionFamily::Activate,
            declaration_identity,
            WorthUiIntentInteractionRouteKind::Confirmation,
            None,
        )
    }

    pub const fn family(&self) -> WorthUiIntentInteractionFamily {
        self.family
    }

    pub fn declaration_identity(&self) -> &str {
        &self.declaration_identity
    }

    pub const fn kind(&self) -> WorthUiIntentInteractionRouteKind {
        self.kind
    }

    pub fn opens_portal(mut self, identity: impl Into<Box<str>>) -> Self {
        let identity = identity.into();
        assert!(
            !identity.trim().is_empty(),
            "opened Portal declaration identity cannot be empty"
        );
        self.opened_portal_identity = Some(identity);
        self
    }

    pub fn opened_portal_identity(&self) -> Option<&str> {
        self.opened_portal_identity.as_deref()
    }

    pub(crate) fn body_atoms(&self) -> Vec<WorthUiArtifactInputBodyAtom> {
        let mut atoms = vec![
            WorthUiArtifactInputBodyAtom::Identifier("interaction".to_owned()),
            WorthUiArtifactInputBodyAtom::Identifier(self.family.as_str().to_owned()),
            WorthUiArtifactInputBodyAtom::Identifier(
                match self.kind {
                    WorthUiIntentInteractionRouteKind::Product => "routes",
                    WorthUiIntentInteractionRouteKind::Confirmation => "confirms",
                }
                .to_owned(),
            ),
            WorthUiArtifactInputBodyAtom::Identifier(self.declaration_identity.to_string()),
        ];
        if let Some(portal) = &self.opened_portal_identity {
            atoms.extend([
                WorthUiArtifactInputBodyAtom::Identifier("opens".to_owned()),
                WorthUiArtifactInputBodyAtom::Identifier("portal".to_owned()),
                WorthUiArtifactInputBodyAtom::Identifier(portal.to_string()),
            ]);
        }
        atoms
    }

    pub(crate) fn from_authored_parts(
        family: WorthUiIntentInteractionFamily,
        declaration_identity: String,
        kind: WorthUiIntentInteractionRouteKind,
        opened_portal_identity: Option<String>,
    ) -> Self {
        Self::new(family, declaration_identity, kind, opened_portal_identity)
    }

    fn new(
        family: WorthUiIntentInteractionFamily,
        declaration_identity: impl Into<Box<str>>,
        kind: WorthUiIntentInteractionRouteKind,
        opened_portal_identity: Option<String>,
    ) -> Self {
        let declaration_identity = declaration_identity.into();
        assert!(
            !declaration_identity.trim().is_empty(),
            "intent route declaration identity cannot be empty"
        );
        Self {
            family,
            declaration_identity,
            kind,
            opened_portal_identity: opened_portal_identity.map(Into::into),
        }
    }
}
