use sha2::{Digest, Sha256};

use crate::filesystem_media::{
    qualification_basis::RootProfileBinding, CapabilitySupport, FilesystemAccessContract,
    FilesystemBackendProfile, FilesystemLocation, MediaCapability,
};

pub(in crate::filesystem_media) fn derive(
    profile: &FilesystemBackendProfile,
    access_contract: FilesystemAccessContract,
) -> RootProfileBinding {
    let support = MediaCapability::ALL.map(|capability| match profile.support(capability) {
        CapabilitySupport::Supported => 1,
        CapabilitySupport::Unsupported => 2,
        CapabilitySupport::Indeterminate => 3,
    });
    let location = match profile.location() {
        FilesystemLocation::Local => 1,
        FilesystemLocation::Remote => 2,
        FilesystemLocation::Unknown => 3,
    };
    let access = match access_contract {
        FilesystemAccessContract::CoordinatedServiceAccount => 1,
    };
    let mut digest = Sha256::new();
    for part in [
        profile.filesystem_type().as_bytes(),
        &profile.allocation_granularity().get().to_le_bytes(),
        &[
            location,
            profile.is_removable() as u8,
            profile.is_read_only() as u8,
            access,
        ],
        &support,
    ] {
        digest.update((part.len() as u64).to_le_bytes());
        digest.update(part);
    }
    RootProfileBinding {
        contract_version: super::super::qualification_basis::qualification_contract_version(),
        root_identity: profile.root_identity(),
        volume_identity: profile.volume_identity(),
        profile_digest: digest.finalize().into(),
        backend_build_identity: filesystem_media_build_identity(),
        access_contract,
    }
}

/// Digest of the concrete media implementation sources and build posture.
/// It is rerun-binding evidence only and grants no operational authority.
pub fn filesystem_media_build_identity() -> [u8; 32] {
    let encoded = env!("WORTH_STORE_MEDIA_BUILD_ID").as_bytes();
    let mut identity = [0_u8; 32];
    for (index, pair) in encoded.as_chunks::<2>().0.iter().enumerate() {
        identity[index] = decode_hex(pair[0]) << 4 | decode_hex(pair[1]);
    }
    identity
}

fn decode_hex(byte: u8) -> u8 {
    match byte {
        b'0'..=b'9' => byte - b'0',
        b'a'..=b'f' => byte - b'a' + 10,
        _ => unreachable!("build identity is emitted as lowercase hexadecimal"),
    }
}
