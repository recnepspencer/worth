use crate::capability::MosaicRegionKindId;

use super::{MosaicRegionAcceptedRegistrationProof, MosaicRegionKindDescriptor};

#[path = "authored_scroll_succession.rs"]
mod authored_scroll_succession;

pub(crate) use authored_scroll_succession::UiAuthoredScrollRegionClauses;
pub use authored_scroll_succession::{UiAuthoredScrollRegionCause, UiAuthoredScrollRegionDenial};

/// Canonical frozen mosaic region kind capability index.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct FrozenMosaicRegionCapabilities {
    descriptors: Vec<MosaicRegionKindDescriptor>,
    seam_paint: Option<super::MosaicSeamPaintContract>,
}

impl FrozenMosaicRegionCapabilities {
    #[cfg(test)]
    pub(crate) fn empty() -> Self {
        Self {
            descriptors: Vec::new(),
            seam_paint: None,
        }
    }

    pub(crate) fn from_accepted_descriptors(
        mut descriptors: Vec<MosaicRegionKindDescriptor>,
        accepted_regions: &MosaicRegionAcceptedRegistrationProof,
        seam_paint: Option<super::MosaicSeamPaintContract>,
    ) -> Self {
        descriptors.retain(|descriptor| accepted_regions.admits(descriptor));
        descriptors.sort_by(|left, right| left.id().cmp(right.id()));
        let accepted_ids = descriptors
            .iter()
            .map(|descriptor| descriptor.id())
            .collect::<Vec<_>>();
        let seam_paint = seam_paint
            .filter(|contract| contract.regions().iter().eq(accepted_ids.iter().copied()));
        Self {
            descriptors,
            seam_paint,
        }
    }

    pub fn is_empty(&self) -> bool {
        self.descriptors.is_empty()
    }

    pub fn len(&self) -> usize {
        self.descriptors.len()
    }

    pub fn descriptors(&self) -> &[MosaicRegionKindDescriptor] {
        &self.descriptors
    }

    /// The appearance roles every region's declared scroll chrome paints with.
    ///
    /// A theme has to admit these alongside the roles nodes consume: a frame
    /// that derives chrome resolves its track and thumb against the same
    /// binding, and a role the binding never admitted refuses the frame.
    pub fn scroll_chrome_role_identities(
        &self,
    ) -> impl Iterator<Item = worth_ui_dsl::UiAppearanceRoleIdentity> + '_ {
        self.descriptors
            .iter()
            .filter_map(MosaicRegionKindDescriptor::scroll_chrome)
            .flat_map(|chrome| [chrome.track_role(), chrome.thumb_role()])
    }

    pub fn seam_paint(&self) -> Option<&super::MosaicSeamPaintContract> {
        self.seam_paint.as_ref()
    }

    pub fn get(&self, id: &MosaicRegionKindId) -> Option<&MosaicRegionKindDescriptor> {
        self.descriptors
            .binary_search_by(|descriptor| descriptor.id().cmp(id))
            .ok()
            .map(|index| &self.descriptors[index])
    }

    pub(crate) fn runtime_service_support(&self) -> crate::capability::UiRuntimeServiceSupport {
        use crate::capability::{MosaicScrollOwnership as Ownership, UiRuntimeServiceFamily};

        let scroll_declared = self.descriptors.iter().any(|descriptor| {
            matches!(
                descriptor.scroll_ownership(),
                Some(Ownership::RegionOwned | Ownership::SurfaceOwned | Ownership::ViewportOwned)
            )
        });
        if scroll_declared {
            crate::capability::UiRuntimeServiceSupport::none_installed()
                .with_installed(UiRuntimeServiceFamily::Scroll)
        } else {
            crate::capability::UiRuntimeServiceSupport::none_installed()
        }
    }

    pub(crate) fn digest_basis(&self) -> u64 {
        let regions = self.region_kind_digest_basis();
        self.seam_paint.as_ref().map_or_else(
            || fold_bytes(regions, b"no_seam_paint"),
            |contract| fold_seam_paint_contract(regions, contract),
        )
    }

    pub(crate) fn region_kind_digest_basis(&self) -> u64 {
        self.descriptors
            .iter()
            .fold(0xd46a_193f_70c5_8ab1, fold_mosaic_region_descriptor)
    }

    pub(crate) fn seam_paint_digest_basis(&self) -> u64 {
        self.seam_paint.as_ref().map_or(0, |contract| {
            fold_seam_paint_contract(0x6177_2bbf_2e43_087d, contract)
        })
    }
}

impl MosaicRegionKindDescriptor {
    /// The complete region-kind basis shared with the frozen capability catalog.
    pub(crate) fn digest_basis(&self) -> u64 {
        fold_mosaic_region_descriptor(0xd46a_193f_70c5_8ab1, self)
    }
}

fn fold_seam_paint_contract(accumulator: u64, contract: &super::MosaicSeamPaintContract) -> u64 {
    let with_regions = contract.regions().iter().fold(
        fold_bytes(accumulator, b"seam_regions"),
        |digest, region| fold_bytes(digest, region.as_str().as_bytes()),
    );
    let with_edges = contract.shared_edges().iter().fold(
        fold_bytes(with_regions, b"seam_edges"),
        |digest, edge| {
            edge.endpoints().into_iter().fold(digest, |value, id| {
                fold_bytes(value, id.as_str().as_bytes())
            })
        },
    );
    let with_owners = contract.owners().iter().fold(with_edges, |digest, owner| {
        fold_bytes(digest, owner.owner().as_str().as_bytes())
    });
    contract
        .exterior_corners()
        .iter()
        .fold(with_owners, |digest, corner| {
            fold_bytes(
                fold_bytes(digest, corner.region().as_str().as_bytes()),
                &[corner.posture() as u8],
            )
        })
}

fn fold_mosaic_region_descriptor(accumulator: u64, descriptor: &MosaicRegionKindDescriptor) -> u64 {
    let with_id = fold_bytes(accumulator, descriptor.id().as_str().as_bytes());
    let with_role = fold_bytes(with_id, descriptor.role().digest_basis().as_bytes());
    let with_sizing = fold_optional_str(
        with_role,
        descriptor
            .sizing_behavior()
            .map(|sizing_behavior| sizing_behavior.digest_basis()),
    );
    let with_scroll = fold_optional_str(
        with_sizing,
        descriptor
            .scroll_ownership()
            .map(|scroll_ownership| scroll_ownership.digest_basis()),
    );
    let with_focus = fold_optional_str(
        with_scroll,
        descriptor
            .focus_scope()
            .map(|focus_scope| focus_scope.digest_basis()),
    );
    let with_child = fold_optional_str(
        with_focus,
        descriptor
            .child_rule()
            .map(|child_rule| child_rule.digest_basis()),
    );
    let with_surface_classes = descriptor.allowed_surface_classes().iter().fold(
        fold_bytes(with_child, b"allowed_surface_classes"),
        |accumulator, surface_class| fold_list_item(accumulator, &surface_class.digest_basis()),
    );
    let with_persistence = fold_optional_str(
        with_surface_classes,
        descriptor
            .persistence()
            .map(|persistence| persistence.digest_basis()),
    );
    let with_clipping = fold_optional_str(
        with_persistence,
        descriptor
            .clipping()
            .map(|clipping| clipping.digest_basis()),
    );
    let with_hit_test = fold_optional_str(
        with_clipping,
        descriptor
            .hit_test()
            .map(|hit_test| hit_test.digest_basis()),
    );
    let with_chrome = fold_scroll_chrome(with_hit_test, descriptor.scroll_chrome());
    let with_line_extent = match descriptor.scroll_line_extent() {
        Some(extent) => fold_bytes(
            fold_bytes(with_chrome, b"scroll_line_extent"),
            &extent.logical_points_value().to_be_bytes(),
        ),
        None => fold_bytes(with_chrome, b"no_scroll_line_extent"),
    };
    fold_optional_str(with_line_extent, descriptor.label())
}

/// Fold a region's declared scroll chrome, which paints two named parts over
/// the axes the region shows bars for.
fn fold_scroll_chrome(accumulator: u64, chrome: Option<&super::UiScrollChromeContract>) -> u64 {
    let Some(contract) = chrome else {
        return fold_bytes(accumulator, b"no_scroll_chrome");
    };
    let with_axes = fold_bytes(
        fold_bytes(accumulator, b"scroll_chrome"),
        &[contract.axes() as u8],
    );
    let with_track = fold_list_item(with_axes, contract.track_role().as_str());
    fold_list_item(with_track, contract.thumb_role().as_str())
}

fn fold_list_item(accumulator: u64, value: &str) -> u64 {
    fold_bytes(fold_bytes(accumulator, b"item"), value.as_bytes())
}

fn fold_optional_str(accumulator: u64, value: Option<&str>) -> u64 {
    match value {
        Some(value) => fold_bytes(fold_bytes(accumulator, b"some"), value.as_bytes()),
        None => fold_bytes(accumulator, b"none"),
    }
}

fn fold_bytes(mut accumulator: u64, bytes: &[u8]) -> u64 {
    for byte in bytes {
        accumulator ^= u64::from(*byte);
        accumulator = accumulator.wrapping_mul(0x0000_0100_0000_01b3);
    }
    accumulator
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::capability::registry::mosaic_region::descriptor::{
        UiScrollAxisSupport, UiScrollChromeContract,
    };
    use crate::capability::{MosaicRegionRole, MosaicSeamPaintContract};

    #[test]
    fn scroll_chrome_role_identities_name_every_declared_track_and_thumb() {
        let role = |name: &str| worth_ui_dsl::UiAppearanceRoleIdentity::new(name).unwrap();
        let plain = MosaicRegionKindDescriptor::new(
            MosaicRegionKindId::new("region.plain").unwrap(),
            MosaicRegionRole::primary(),
        );
        let scrolled = MosaicRegionKindDescriptor::new(
            MosaicRegionKindId::new("region.scrolled").unwrap(),
            MosaicRegionRole::primary(),
        )
        .with_scroll_chrome(
            UiScrollChromeContract::new(
                UiScrollAxisSupport::Block,
                role("chrome.track"),
                role("chrome.thumb"),
            )
            .unwrap(),
        );
        let capabilities = FrozenMosaicRegionCapabilities {
            descriptors: vec![plain, scrolled],
            seam_paint: None,
        };

        assert_eq!(
            capabilities
                .scroll_chrome_role_identities()
                .collect::<Vec<_>>(),
            vec![role("chrome.track"), role("chrome.thumb")]
        );
    }

    #[test]
    fn seam_digest_changes_without_changing_region_kind_digest() {
        let region_id = MosaicRegionKindId::new("region.primary").unwrap();
        let descriptor =
            MosaicRegionKindDescriptor::new(region_id.clone(), MosaicRegionRole::primary());
        let without_seam = FrozenMosaicRegionCapabilities {
            descriptors: vec![descriptor.clone()],
            seam_paint: None,
        };
        let with_seam = FrozenMosaicRegionCapabilities {
            descriptors: vec![descriptor],
            seam_paint: Some(MosaicSeamPaintContract::admit([region_id], [], [], []).unwrap()),
        };

        assert_eq!(
            without_seam.region_kind_digest_basis(),
            with_seam.region_kind_digest_basis()
        );
        assert_ne!(
            without_seam.seam_paint_digest_basis(),
            with_seam.seam_paint_digest_basis()
        );
    }
}
