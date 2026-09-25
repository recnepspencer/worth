/// Product authority carried by the existing provider-session protocol.
///
/// Standalone managed runs have no application product. Application runs must
/// replace that posture with the exact selected-product lease before the
/// provider can mint a session token.
pub(crate) enum WorthQueryProviderProductAffinity {
    Standalone,
    Application(crate::basis::WorthQueryProductBranchLease),
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(in crate::domain_computation) enum WorthQueryProviderTerminalProductAffinity {
    Standalone,
    Application(crate::basis::WorthQueryProductBranchReadIdentity),
}

impl WorthQueryProviderProductAffinity {
    pub(super) const fn standalone() -> Self {
        Self::Standalone
    }

    pub(super) const fn application(product: crate::basis::WorthQueryProductBranchLease) -> Self {
        Self::Application(product)
    }

    pub(super) fn terminal(&self) -> WorthQueryProviderTerminalProductAffinity {
        match self {
            Self::Standalone => WorthQueryProviderTerminalProductAffinity::Standalone,
            Self::Application(product) => WorthQueryProviderTerminalProductAffinity::Application(
                crate::basis::WorthQueryProductBranchReadIdentity::from_observation(
                    product.observation(),
                ),
            ),
        }
    }
}

impl WorthQueryProviderTerminalProductAffinity {
    pub(in crate::domain_computation) fn application_product_identity(
        &self,
    ) -> Option<&crate::basis::WorthQueryProductBranchReadIdentity> {
        match self {
            Self::Standalone => None,
            Self::Application(product) => Some(product),
        }
    }
}
