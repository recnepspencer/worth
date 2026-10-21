//! Descriptive association of a native performance with its producing build.

/// Original producing meaning, carried through native publication and recovery.
/// This value grants no publication, currentness or recovery authority.
#[derive(
    Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd, serde::Serialize, serde::Deserialize,
)]
pub struct RequiredOutputProducingImplementation {
    build: [u8; 16],
    edition: [u8; 32],
}

impl RequiredOutputProducingImplementation {
    pub fn new(
        build: worth_foundational::facade::ApplicationBuildIdentity,
        edition: [u8; 32],
    ) -> Self {
        Self {
            build: build.bytes(),
            edition,
        }
    }

    /// Decode descriptive wire fields; native admission remains separate.
    pub fn from_checkpoint(build: [u8; 16], edition: [u8; 32]) -> Result<Self, &'static str> {
        if build == [0; 16] {
            return Err("required output has no producing build identity");
        }
        Ok(Self { build, edition })
    }

    pub const fn build_identity(self) -> [u8; 16] {
        self.build
    }

    pub const fn edition(self) -> [u8; 32] {
        self.edition
    }

    pub(crate) fn valid(self) -> bool {
        self.build != [0; 16]
    }
}
