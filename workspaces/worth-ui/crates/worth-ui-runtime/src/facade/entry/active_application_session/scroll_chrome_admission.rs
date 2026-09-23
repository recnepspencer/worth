//! Admitting one Scroll region occurrence's declared chrome.
//!
//! `UiScrollChromeContract` carries role identities and an axis declaration and
//! nothing else: it holds no appearance registry and knows nothing of the
//! region's scroll ownership, so it cannot answer whether the roles it names
//! exist or whether the region owns what the chrome would travel over. This is
//! the site where those two questions are asked, and it is asked before any
//! geometry is derived, so an inadmissible declaration paints nothing rather
//! than painting something wrong.

use crate::capability::{
    FrozenAppearanceRoleCapabilities, MosaicScrollOwnership, UiScrollChromeContract,
    UiScrollChromeContractDenial,
};
use crate::runtime::planning::execution_plan_input::WorthUiPlanOrdinaryMeaning;
use crate::runtime::scroll::chrome::{
    UiScrollAdmittedChrome, UiScrollChromeAdmissionDenial, UiScrollChromeAxisSupport,
    UiScrollChromeMetrics,
};

/// Why a Scroll owner presents no admitted chrome.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(in crate::facade::entry) enum UiScrollDeclaredChromeDenial {
    /// The owner is a surface or viewport owner, or its plan occurrence names
    /// no region-kind descriptor, so nothing declares chrome for it at all.
    NoRegionDeclaration,
    /// The region declares no chrome. Most regions declare none; this is an
    /// absence, not a fault.
    NoChromeDeclared,
    /// The declaration is inadmissible where it meets the appearance role
    /// registry or the region's scroll ownership.
    Contract(UiScrollChromeContractDenial),
    /// The Scroll runtime refused the admitted form of the contract.
    Admission(UiScrollChromeAdmissionDenial),
}

impl super::super::WorthUiActiveApplicationSession {
    /// The chrome `owner` declares, admitted into the runtime-side form the
    /// Scroll chrome geometry consumes.
    ///
    /// This is the single adapter point from the declared contract, and
    /// therefore the single place the contract's two deferred questions are
    /// answered: whether the named roles are registered, and whether this
    /// region owns the scrolling its chrome would show.
    pub(in crate::facade::entry) fn admitted_scroll_chrome(
        &self,
        owner: crate::runtime::scroll::UiScrollOwnerIdentity,
    ) -> Result<UiScrollAdmittedChrome, UiScrollDeclaredChromeDenial> {
        let meaning = self
            .declared_scroll_region_descriptor(owner)
            .ok_or(UiScrollDeclaredChromeDenial::NoRegionDeclaration)?;
        let WorthUiPlanOrdinaryMeaning::Layout(layout) = meaning.as_ref() else {
            return Err(UiScrollDeclaredChromeDenial::NoRegionDeclaration);
        };
        let descriptor = layout
            .region_descriptor()
            .ok_or(UiScrollDeclaredChromeDenial::NoRegionDeclaration)?;
        let contract = descriptor
            .scroll_chrome()
            .ok_or(UiScrollDeclaredChromeDenial::NoChromeDeclared)?;
        admit_region_chrome(
            contract,
            descriptor.scroll_ownership(),
            self.capabilities().appearance_roles(),
        )
    }
}

/// Admit one declared contract against the region's scroll ownership and the
/// frozen appearance role registry.
///
/// Ownership is asked first: a region that owns no scrolling has no axis for
/// chrome to travel over, so the roles it names are beside the point.
pub(super) fn admit_region_chrome(
    contract: &UiScrollChromeContract,
    ownership: Option<&MosaicScrollOwnership>,
    registered_roles: &FrozenAppearanceRoleCapabilities,
) -> Result<UiScrollAdmittedChrome, UiScrollDeclaredChromeDenial> {
    if !owns_its_own_scrolling(ownership) {
        return Err(UiScrollDeclaredChromeDenial::Contract(
            UiScrollChromeContractDenial::AxisNotOwned,
        ));
    }
    for role in [contract.track_role(), contract.thumb_role()] {
        if registered_roles.get(&role).is_none() {
            return Err(UiScrollDeclaredChromeDenial::Contract(
                UiScrollChromeContractDenial::UnregisteredRole,
            ));
        }
    }
    UiScrollAdmittedChrome::admit_declared_chrome(
        admitted_axis_support(contract.axes()),
        contract.track_role(),
        contract.thumb_role(),
        UiScrollChromeMetrics::declared(),
    )
    .map_err(UiScrollDeclaredChromeDenial::Admission)
}

/// Whether the region occurrence that declared this chrome is the authority
/// that scrolls.
///
/// Only `RegionOwned` resolves to a Scroll region owner; a surface-owned or
/// viewport-owned region scrolls under an authority that is not this region, so
/// chrome declared here would report an axis this region does not own. Scroll
/// ownership carries no per-axis granularity, so the question this answers is
/// per region, not per declared axis.
const fn owns_its_own_scrolling(ownership: Option<&MosaicScrollOwnership>) -> bool {
    matches!(ownership, Some(MosaicScrollOwnership::RegionOwned))
}

/// The declared axis support as the Scroll runtime names it. The declaration
/// and the runtime keep separate enums on purpose, so this is the only place
/// the two vocabularies meet.
pub(super) fn admitted_axis_support(
    axes: crate::capability::UiScrollAxisSupport,
) -> UiScrollChromeAxisSupport {
    match axes {
        crate::capability::UiScrollAxisSupport::Inline => UiScrollChromeAxisSupport::Inline,
        crate::capability::UiScrollAxisSupport::Block => UiScrollChromeAxisSupport::Block,
        crate::capability::UiScrollAxisSupport::Both => UiScrollChromeAxisSupport::Both,
    }
}
