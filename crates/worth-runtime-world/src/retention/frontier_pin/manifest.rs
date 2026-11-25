//! The sealed whole-Home frontier a pin captures.
use sha2::{Digest, Sha256};
use worth_foundational::facade::HomeIncarnation;

pub const HOME_ROOT_MANIFEST_VERSION: u16 = 1;
const ENCODED_BYTES: usize = 2 + 8 + 8 + 4 * 16 + 32 + 32;

/// One component's exact coordinate inside a published physical root: the
/// root generation and the component's durable head position in that root.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct PhysicalRootPointer {
    root_generation: u64,
    position: u64,
}

impl PhysicalRootPointer {
    /// Recorded by the Home's physical retention owner from the pinned root.
    pub const fn from_retention_owner(root_generation: u64, position: u64) -> Self {
        Self {
            root_generation,
            position,
        }
    }
    pub const fn root_generation(self) -> u64 {
        self.root_generation
    }
    pub const fn position(self) -> u64 {
        self.position
    }
}

/// Fixed-size whole-Home frontier: Native, World, Signal, the persistent
/// product-name map and the installed program, sealed by one SHA-256 digest.
/// Resolving any product through this manifest uses the product map of this
/// exact frontier, never a newer one.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HomeRootManifest {
    format_version: u16,
    home_incarnation: HomeIncarnation,
    frontier_sequence: u64,
    native_root: PhysicalRootPointer,
    world_root: PhysicalRootPointer,
    signal_root: PhysicalRootPointer,
    product_map_root: PhysicalRootPointer,
    installed_program_digest: [u8; 32],
    manifest_digest: [u8; 32],
}

/// The component coordinates one frontier capture observed.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct HomeFrontierComponents {
    pub native_root: PhysicalRootPointer,
    pub world_root: PhysicalRootPointer,
    pub signal_root: PhysicalRootPointer,
    pub product_map_root: PhysicalRootPointer,
}

impl HomeRootManifest {
    /// Seal a captured frontier. The digest covers every field above it.
    pub fn seal(
        home_incarnation: HomeIncarnation,
        frontier_sequence: u64,
        components: HomeFrontierComponents,
        installed_program_digest: [u8; 32],
    ) -> Self {
        let mut manifest = Self {
            format_version: HOME_ROOT_MANIFEST_VERSION,
            home_incarnation,
            frontier_sequence,
            native_root: components.native_root,
            world_root: components.world_root,
            signal_root: components.signal_root,
            product_map_root: components.product_map_root,
            installed_program_digest,
            manifest_digest: [0; 32],
        };
        manifest.manifest_digest = manifest.digest_fields();
        manifest
    }

    pub const fn format_version(&self) -> u16 {
        self.format_version
    }
    pub const fn home_incarnation(&self) -> HomeIncarnation {
        self.home_incarnation
    }
    pub const fn frontier_sequence(&self) -> u64 {
        self.frontier_sequence
    }
    pub const fn native_root(&self) -> PhysicalRootPointer {
        self.native_root
    }
    pub const fn world_root(&self) -> PhysicalRootPointer {
        self.world_root
    }
    pub const fn signal_root(&self) -> PhysicalRootPointer {
        self.signal_root
    }
    pub const fn product_map_root(&self) -> PhysicalRootPointer {
        self.product_map_root
    }
    pub const fn installed_program_digest(&self) -> [u8; 32] {
        self.installed_program_digest
    }
    pub const fn manifest_digest(&self) -> [u8; 32] {
        self.manifest_digest
    }

    /// Canonical fixed-size encoding, digest included.
    pub fn encode(&self) -> Vec<u8> {
        let mut bytes = self.encode_fields();
        bytes.extend_from_slice(&self.manifest_digest);
        bytes
    }

    /// Decode and verify the seal; any mismatch is refused.
    pub fn decode(bytes: &[u8]) -> Option<Self> {
        if bytes.len() != ENCODED_BYTES {
            return None;
        }
        let u64_at =
            |at: usize| u64::from_le_bytes(bytes[at..at + 8].try_into().unwrap_or_default());
        let pointer =
            |at: usize| PhysicalRootPointer::from_retention_owner(u64_at(at), u64_at(at + 8));
        let format_version = u16::from_le_bytes([bytes[0], bytes[1]]);
        if format_version != HOME_ROOT_MANIFEST_VERSION {
            return None;
        }
        let manifest = Self {
            format_version,
            home_incarnation: HomeIncarnation::from_recorded(u64_at(2))?,
            frontier_sequence: u64_at(10),
            native_root: pointer(18),
            world_root: pointer(34),
            signal_root: pointer(50),
            product_map_root: pointer(66),
            installed_program_digest: bytes[82..114].try_into().ok()?,
            manifest_digest: bytes[114..146].try_into().ok()?,
        };
        (manifest.digest_fields() == manifest.manifest_digest).then_some(manifest)
    }

    fn encode_fields(&self) -> Vec<u8> {
        let mut bytes = Vec::with_capacity(ENCODED_BYTES);
        bytes.extend_from_slice(&self.format_version.to_le_bytes());
        bytes.extend_from_slice(&self.home_incarnation.get().to_le_bytes());
        bytes.extend_from_slice(&self.frontier_sequence.to_le_bytes());
        for pointer in [
            self.native_root,
            self.world_root,
            self.signal_root,
            self.product_map_root,
        ] {
            bytes.extend_from_slice(&pointer.root_generation.to_le_bytes());
            bytes.extend_from_slice(&pointer.position.to_le_bytes());
        }
        bytes.extend_from_slice(&self.installed_program_digest);
        bytes
    }

    fn digest_fields(&self) -> [u8; 32] {
        let mut hasher = Sha256::new();
        hasher.update(b"worth.home-root-manifest.v1");
        hasher.update(self.encode_fields());
        hasher.finalize().into()
    }
}
