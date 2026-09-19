use std::sync::atomic::{AtomicU64, Ordering};

static NEXT_MOUNTED_CONTRACT_IDENTITY: AtomicU64 = AtomicU64::new(1);

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct UiMountedContractIdentityExhaustion;

macro_rules! opaque_identity {
    ($name:ident) => {
        #[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
        pub struct $name(u64);

        impl $name {
            #[doc(hidden)]
            pub fn mint_unbound() -> Result<Self, UiMountedContractIdentityExhaustion> {
                next_identity_value().map(Self)
            }

            pub const fn diagnostic_value(self) -> u64 {
                self.0
            }
        }
    };
}

opaque_identity!(UiSemanticSurfaceIdentity);
opaque_identity!(UiHostSurfaceIdentity);
opaque_identity!(UiSurfaceBindingGeneration);
opaque_identity!(UiMountIncarnation);
opaque_identity!(UiMountedInstanceIdentity);
opaque_identity!(UiMountedFrameIdentity);
opaque_identity!(UiMountedContentGeneration);
opaque_identity!(UiMountedPresentationAttemptIdentity);

#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct UiHostPresentationLineageIdentity(u64);

impl UiHostPresentationLineageIdentity {
    pub(crate) const fn from_host_session(host_session_identity: u64) -> Option<Self> {
        if host_session_identity == 0 {
            None
        } else {
            Some(Self(host_session_identity))
        }
    }

    #[cfg(feature = "certification-construction")]
    #[doc(hidden)]
    pub const fn from_certification_host_session(host_session_identity: u64) -> Option<Self> {
        Self::from_host_session(host_session_identity)
    }

    pub const fn diagnostic_value(self) -> u64 {
        self.0
    }
}

#[doc(hidden)]
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct UiMountedNodeReceiptIssuer {
    frame: UiMountedFrameIdentity,
    nonce: u64,
}

#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct UiMountedNodeReceiptIdentity {
    frame: UiMountedFrameIdentity,
    mounted_instance: UiMountedInstanceIdentity,
    issuer_nonce: u64,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct UiMountedNodeReceiptAffinity {
    issuer: UiMountedNodeReceiptIssuer,
}

impl UiMountedNodeReceiptIssuer {
    #[doc(hidden)]
    pub fn mint_for(
        frame: UiMountedFrameIdentity,
    ) -> Result<Self, UiMountedContractIdentityExhaustion> {
        Ok(Self {
            frame,
            nonce: next_identity_value()?,
        })
    }

    #[doc(hidden)]
    pub const fn receipt_for(
        self,
        mounted_instance: UiMountedInstanceIdentity,
    ) -> UiMountedNodeReceiptIdentity {
        UiMountedNodeReceiptIdentity {
            frame: self.frame,
            mounted_instance,
            issuer_nonce: self.nonce,
        }
    }

    #[doc(hidden)]
    pub const fn frame_identity(self) -> UiMountedFrameIdentity {
        self.frame
    }

    pub(crate) const fn issuer_nonce(self) -> u64 {
        self.nonce
    }

    #[doc(hidden)]
    pub const fn receipt_affinity(self) -> UiMountedNodeReceiptAffinity {
        UiMountedNodeReceiptAffinity { issuer: self }
    }
}

impl UiMountedNodeReceiptIdentity {
    #[doc(hidden)]
    pub fn mint_unbound() -> Result<Self, UiMountedContractIdentityExhaustion> {
        let frame = UiMountedFrameIdentity::mint_unbound()?;
        let mounted_instance = UiMountedInstanceIdentity::mint_unbound()?;
        UiMountedNodeReceiptIssuer::mint_for(frame)
            .map(|issuer| issuer.receipt_for(mounted_instance))
    }

    pub const fn diagnostic_value(self) -> u64 {
        self.issuer_nonce.rotate_left(17)
            ^ self.frame.diagnostic_value().rotate_left(37)
            ^ self
                .mounted_instance
                .diagnostic_value()
                .wrapping_mul(0x9e37_79b9_7f4a_7c15)
    }

    pub(crate) const fn frame(self) -> UiMountedFrameIdentity {
        self.frame
    }

    pub const fn mounted_instance(self) -> UiMountedInstanceIdentity {
        self.mounted_instance
    }
}

impl UiMountedNodeReceiptAffinity {
    pub(crate) const fn from_receipt(receipt: UiMountedNodeReceiptIdentity) -> Self {
        Self {
            issuer: UiMountedNodeReceiptIssuer {
                frame: receipt.frame,
                nonce: receipt.issuer_nonce,
            },
        }
    }

    #[doc(hidden)]
    pub const fn rebind_node_receipt(
        self,
        receipt: UiMountedNodeReceiptIdentity,
    ) -> UiMountedNodeReceiptIdentity {
        self.issuer.receipt_for(receipt.mounted_instance())
    }

    #[doc(hidden)]
    pub const fn rebind_realized_region(
        self,
        region: crate::UiHostRealizedRegion,
    ) -> crate::UiHostRealizedRegion {
        crate::UiHostRealizedRegion::observed_by_host(
            self.rebind_node_receipt(region.mounted_receipt()),
            crate::UiHostRealizedGeometry::observed_by_host(region.bounds(), region.clip()),
            crate::UiHostRealizedOrdering::observed_by_host(
                region.semantic_order(),
                region.participation(),
            ),
        )
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum UiHostSurfacePresentationMode {
    NativeDisplay,
    RecordOnly,
}

fn next_identity_value() -> Result<u64, UiMountedContractIdentityExhaustion> {
    NEXT_MOUNTED_CONTRACT_IDENTITY
        .fetch_update(Ordering::Relaxed, Ordering::Relaxed, |current| {
            current.checked_add(1)
        })
        .map_err(|_| UiMountedContractIdentityExhaustion)
}
