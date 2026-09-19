#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct UiMountedNodeAppearanceAttribution {
    frame: crate::UiMountedFrameIdentity,
    issuer_nonce: u64,
    identity: u64,
    revision: u64,
}

impl UiMountedNodeAppearanceAttribution {
    #[doc(hidden)]
    pub const fn from_runtime_mounting(
        issuer: crate::UiMountedNodeReceiptIssuer,
        identity: u64,
        revision: u64,
    ) -> Option<Self> {
        if identity == 0 || revision == 0 {
            None
        } else {
            Some(Self {
                frame: issuer.frame_identity(),
                issuer_nonce: issuer.issuer_nonce(),
                identity,
                revision,
            })
        }
    }

    #[doc(hidden)]
    pub fn matches_issuer(self, issuer: crate::UiMountedNodeReceiptIssuer) -> bool {
        self.frame == issuer.frame_identity() && self.issuer_nonce == issuer.issuer_nonce()
    }

    pub const fn identity(self) -> u64 {
        self.identity
    }

    pub const fn revision(self) -> u64 {
        self.revision
    }
}
