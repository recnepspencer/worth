use crate::capability::{
    MosaicRegionKindDescriptor, MosaicRegionKindId, MosaicScrollOwnership, UiScrollChromeContract,
    UiScrollChromeContractDenial, UiScrollLineExtent, UiScrollLineExtentDenial,
};

use super::FrozenMosaicRegionCapabilities;

/// Why authored scroll clauses could not reach the region descriptor they name.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum UiAuthoredScrollRegionCause {
    /// The `scroll` block names something that is not a well formed region kind
    /// identity, so no region could be looked up at all.
    RegionIdentityMalformed,
    /// A `track` or `thumb` clause names something that is not a well formed
    /// appearance role identity.
    AppearanceRoleIdentityMalformed,
    /// The named region kind is not registered. An authored line extent or
    /// chrome contract would otherwise describe a region that does not exist.
    UnregisteredRegion,
    /// The authored `line_extent` is not an admissible extent.
    LineExtent(UiScrollLineExtentDenial),
    /// The authored `chrome`, `track` and `thumb` are not an admissible
    /// contract for this region.
    Chrome(UiScrollChromeContractDenial),
    /// The authored source and the registered descriptor state different line
    /// extents for the same region. Two sources naming one region disagree, and
    /// neither silently wins.
    LineExtentDisagreement {
        /// The extent the `.wui` source states, in logical points.
        authored: u16,
        /// The extent the registered descriptor states, in logical points.
        registered: u16,
    },
    /// The authored source and the registered descriptor state different chrome
    /// for the same region.
    ChromeDisagreement,
}

/// A refused lowering of one authored `scroll` block's region clauses.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct UiAuthoredScrollRegionDenial {
    declaration_index: usize,
    cause: UiAuthoredScrollRegionCause,
}

impl UiAuthoredScrollRegionDenial {
    pub(crate) const fn new(declaration_index: usize, cause: UiAuthoredScrollRegionCause) -> Self {
        Self {
            declaration_index,
            cause,
        }
    }

    /// Which authored service declaration carried the refused clauses.
    pub const fn declaration_index(&self) -> usize {
        self.declaration_index
    }

    pub const fn cause(&self) -> UiAuthoredScrollRegionCause {
        self.cause
    }
}

/// One authored `scroll` block's region-level clauses, already typed.
///
/// This carries no policy: the app-wide scroll policy is lowered elsewhere.
/// These are the clauses that belong to one named region and to nothing else.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct UiAuthoredScrollRegionClauses {
    declaration_index: usize,
    region: MosaicRegionKindId,
    line_extent: Option<UiScrollLineExtent>,
    chrome: Option<UiScrollChromeContract>,
}

impl UiAuthoredScrollRegionClauses {
    pub(crate) const fn new(
        declaration_index: usize,
        region: MosaicRegionKindId,
        line_extent: Option<UiScrollLineExtent>,
        chrome: Option<UiScrollChromeContract>,
    ) -> Self {
        Self {
            declaration_index,
            region,
            line_extent,
            chrome,
        }
    }

    pub(crate) const fn region(&self) -> &MosaicRegionKindId {
        &self.region
    }

    pub(crate) fn chrome(&self) -> Option<&UiScrollChromeContract> {
        self.chrome.as_ref()
    }

    pub(crate) const fn denied(
        &self,
        cause: UiAuthoredScrollRegionCause,
    ) -> UiAuthoredScrollRegionDenial {
        UiAuthoredScrollRegionDenial::new(self.declaration_index, cause)
    }

    /// Fold these clauses onto the registered descriptor, or refuse.
    ///
    /// `Ok(None)` means the descriptor already states exactly what the source
    /// states, so no succession is owed.
    fn lower_onto(
        &self,
        registered: &MosaicRegionKindDescriptor,
    ) -> Result<Option<MosaicRegionKindDescriptor>, UiAuthoredScrollRegionDenial> {
        let mut lowered = None;
        if let Some(authored) = self.line_extent {
            match registered.scroll_line_extent() {
                Some(stated) if stated != authored => {
                    return Err(
                        self.denied(UiAuthoredScrollRegionCause::LineExtentDisagreement {
                            authored: authored.logical_points_value(),
                            registered: stated.logical_points_value(),
                        }),
                    )
                }
                Some(_) => {}
                None => {
                    lowered = Some(
                        lowered
                            .unwrap_or_else(|| registered.clone())
                            .with_scroll_line_extent(authored),
                    );
                }
            }
        }
        if let Some(authored) = self.chrome.as_ref() {
            if !owns_scrollable_content(registered.scroll_ownership()) {
                return Err(self.denied(UiAuthoredScrollRegionCause::Chrome(
                    UiScrollChromeContractDenial::AxisNotOwned,
                )));
            }
            match registered.scroll_chrome() {
                Some(stated) if stated != authored => {
                    return Err(self.denied(UiAuthoredScrollRegionCause::ChromeDisagreement))
                }
                Some(_) => {}
                None => {
                    lowered = Some(
                        lowered
                            .unwrap_or_else(|| registered.clone())
                            .with_scroll_chrome(authored.clone()),
                    );
                }
            }
        }
        Ok(lowered)
    }
}

/// Whether a region owns content a scrollbar could travel over.
const fn owns_scrollable_content(ownership: Option<&MosaicScrollOwnership>) -> bool {
    matches!(
        ownership,
        Some(
            MosaicScrollOwnership::RegionOwned
                | MosaicScrollOwnership::SurfaceOwned
                | MosaicScrollOwnership::ViewportOwned
        )
    )
}

impl FrozenMosaicRegionCapabilities {
    /// Prepare the successor region set the authored scroll clauses ask for.
    ///
    /// `Ok(None)` means every clause already agreed with its registered
    /// descriptor, so the frozen set stands unchanged.
    pub(crate) fn prepare_authored_scroll_succession(
        &self,
        clauses: &[UiAuthoredScrollRegionClauses],
    ) -> Result<Option<Self>, UiAuthoredScrollRegionDenial> {
        let mut successor: Option<Vec<MosaicRegionKindDescriptor>> = None;
        for clause in clauses {
            let current = successor.as_ref().unwrap_or(&self.descriptors);
            let index = current
                .binary_search_by(|descriptor| descriptor.id().cmp(clause.region()))
                .map_err(|_| clause.denied(UiAuthoredScrollRegionCause::UnregisteredRegion))?;
            let Some(lowered) = clause.lower_onto(&current[index])? else {
                continue;
            };
            successor.get_or_insert_with(|| self.descriptors.clone())[index] = lowered;
        }
        Ok(successor.map(|descriptors| Self {
            descriptors,
            seam_paint: self.seam_paint.clone(),
        }))
    }
}
