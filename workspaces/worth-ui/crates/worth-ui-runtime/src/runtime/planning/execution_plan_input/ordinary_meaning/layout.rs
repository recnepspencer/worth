use crate::capability::{
    MosaicPlacementPolicyDescriptor, MosaicRegionKindDescriptor, MosaicSizingContractDescriptor,
    SurfaceDescriptor,
};

use super::digest::fold;

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) enum WorthUiLayoutPlanMeaning {
    Region {
        descriptor: MosaicRegionKindDescriptor,
        sizing_contract: Option<MosaicSizingContractDescriptor>,
        child_range_identity: Option<String>,
    },
    Surface {
        descriptor: SurfaceDescriptor,
        placement_policy: Option<MosaicPlacementPolicyDescriptor>,
        child_range_identity: Option<String>,
    },
}

impl WorthUiLayoutPlanMeaning {
    pub(crate) fn region(
        descriptor: MosaicRegionKindDescriptor,
        sizing_contract: Option<MosaicSizingContractDescriptor>,
        child_range_identity: Option<String>,
    ) -> Self {
        Self::Region {
            descriptor,
            sizing_contract,
            child_range_identity,
        }
    }

    pub(crate) fn surface(
        descriptor: SurfaceDescriptor,
        placement_policy: Option<MosaicPlacementPolicyDescriptor>,
        child_range_identity: Option<String>,
    ) -> Self {
        Self::Surface {
            descriptor,
            placement_policy,
            child_range_identity,
        }
    }

    pub(crate) fn child_range_identity(&self) -> Option<&str> {
        match self {
            Self::Region {
                child_range_identity,
                ..
            }
            | Self::Surface {
                child_range_identity,
                ..
            } => child_range_identity.as_deref(),
        }
    }

    pub(crate) const fn region_descriptor(&self) -> Option<&MosaicRegionKindDescriptor> {
        match self {
            Self::Region { descriptor, .. } => Some(descriptor),
            Self::Surface { .. } => None,
        }
    }

    pub(crate) fn semantic_digest(&self) -> u64 {
        match self {
            Self::Region {
                descriptor,
                sizing_contract,
                ..
            } => {
                let digest = fold(0x7265_6769_6f6e_0001, descriptor.digest_basis());
                sizing_contract.as_ref().map_or(fold(digest, 0), |sizing| {
                    fold(fold(digest, 1), sizing.digest_basis())
                })
            }
            Self::Surface {
                descriptor,
                placement_policy,
                ..
            } => {
                let digest = fold(0x7375_7266_6163_6501, descriptor.digest_basis());
                placement_policy
                    .as_ref()
                    .map_or(fold(digest, 0), |placement| {
                        fold(fold(digest, 1), placement.digest_basis())
                    })
            }
        }
    }
}
