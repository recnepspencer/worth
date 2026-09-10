/// Product authority carried by the existing provider-session protocol.
///
/// Standalone managed runs have no application product. Application runs must
/// replace that posture with the exact selected-product lease before the
/// provider can mint a session token.
pub(crate) enum WorthQueryProviderProductAffinity {
    Standalone,
    Application(crate::basis::WorthQueryProductBranchLease),
}

#[derive(Clone)]
pub(in crate::domain_computation) enum WorthQueryProviderTerminalProductAffinity {
    Standalone,
    Application(
        crate::domain_computation::execution_runtime::product_world::WorthQueryProductPublicationBinding,
    ),
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
                product.publication_binding(),
            ),
        }
    }
}

impl WorthQueryProviderTerminalProductAffinity {
    pub(in crate::domain_computation) fn application_product(
        &self,
    ) -> Option<
        &crate::domain_computation::execution_runtime::product_world::WorthQueryProductPublicationBinding,
    >{
        match self {
            Self::Standalone => None,
            Self::Application(product) => Some(product),
        }
    }
}

impl PartialEq for WorthQueryProviderTerminalProductAffinity {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Self::Standalone, Self::Standalone) => true,
            (Self::Application(left), Self::Application(right)) => {
                left.observation() == right.observation()
            }
            _ => false,
        }
    }
}

impl Eq for WorthQueryProviderTerminalProductAffinity {}

impl std::fmt::Debug for WorthQueryProviderTerminalProductAffinity {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Standalone => formatter.write_str("Standalone"),
            Self::Application(product) => formatter
                .debug_tuple("Application")
                .field(product.observation())
                .finish(),
        }
    }
}
