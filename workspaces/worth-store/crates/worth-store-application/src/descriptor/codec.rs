//! The landed WSRHOME1 version-2 envelope, with its application binding tag.
use super::{ApplicationHomeDescriptorDenial as Denial, HomeDescriptor};

const MAGIC: &[u8; 8] = b"WSRHOME1";
const VERSION: u16 = 2;
const APPLICATION_BINDING: u16 = 1;
const BUILD_BINDING: u16 = 2;

pub(super) fn size(runtime: &str, profile: &str, binding: usize) -> Option<usize> {
    72_usize
        .checked_add(runtime.len())?
        .checked_add(profile.len())?
        .checked_add(binding)
}

pub(super) fn emit(
    descriptor: &HomeDescriptor,
    runtime: &str,
    profile: &str,
    binding: &[u8],
    bytes: &mut Vec<u8>,
) {
    bytes.extend_from_slice(MAGIC);
    bytes.extend_from_slice(&VERSION.to_le_bytes());
    bytes.extend_from_slice(&descriptor.identity.bytes());
    bytes.extend_from_slice(&descriptor.incarnation.get().to_le_bytes());
    text(runtime, bytes);
    text(profile, bytes);
    bytes.extend_from_slice(&2_u16.to_le_bytes());
    bytes.extend_from_slice(&APPLICATION_BINDING.to_le_bytes());
    bytes.extend_from_slice(&(binding.len() as u32).to_le_bytes());
    bytes.extend_from_slice(binding);
    bytes.extend_from_slice(&BUILD_BINDING.to_le_bytes());
    bytes.extend_from_slice(&16_u32.to_le_bytes());
    bytes.extend_from_slice(&descriptor.build.bytes());
}

fn text(text: &str, bytes: &mut Vec<u8>) {
    bytes.extend_from_slice(&(text.len() as u32).to_le_bytes());
    bytes.extend_from_slice(text.as_bytes());
}

pub(super) fn admit_record(
    bytes: &[u8],
    identity: [u8; 16],
    runtime: &str,
    profile: &str,
    binding: &[u8],
) -> Result<worth_foundational::facade::HomeIncarnation, Denial> {
    let rest = bytes.strip_prefix(MAGIC).ok_or(Denial::Malformed)?;
    let (version, rest) = rest.split_first_chunk::<2>().ok_or(Denial::Malformed)?;
    let version = u16::from_le_bytes(*version);
    if version != VERSION {
        return Err(Denial::UnsupportedVersion { observed: version });
    }
    let (found_identity, rest) = rest.split_first_chunk::<16>().ok_or(Denial::Malformed)?;
    if *found_identity != identity {
        return Err(Denial::IdentityMismatch);
    }
    let (incarnation, rest) = rest.split_first_chunk::<8>().ok_or(Denial::Malformed)?;
    let incarnation = worth_foundational::facade::HomeIncarnation::from_recorded(
        u64::from_le_bytes(*incarnation),
    )
    .ok_or(Denial::Malformed)?;
    let (found_runtime, rest) = take_text(rest)?;
    let (found_profile, rest) = take_text(rest)?;
    let (count, mut rest) = rest.split_first_chunk::<2>().ok_or(Denial::Malformed)?;
    let mut application = None;
    let mut build = None;
    for _ in 0..u16::from_le_bytes(*count) {
        let (tag, next) = rest.split_first_chunk::<2>().ok_or(Denial::Malformed)?;
        let (length, next) = next.split_first_chunk::<4>().ok_or(Denial::Malformed)?;
        let length = u32::from_le_bytes(*length) as usize;
        let value = next.get(..length).ok_or(Denial::Malformed)?;
        rest = &next[length..];
        match u16::from_le_bytes(*tag) {
            APPLICATION_BINDING if application.is_none() => application = Some(value),
            BUILD_BINDING if build.is_none() => build = Some(value),
            APPLICATION_BINDING | BUILD_BINDING => return Err(Denial::Malformed),
            tag => return Err(Denial::UnsupportedBinding { tag }),
        }
    }
    if !rest.is_empty() {
        return Err(Denial::Malformed);
    }
    let application = application.ok_or(Denial::ApplicationBindingMissing)?;
    let build = build.ok_or(Denial::BuildBindingMissing)?;
    if build.len() != 16 || build == [0; 16] {
        return Err(Denial::Malformed);
    }
    // A prior implementation build is lineage, never authored program meaning.
    if found_runtime != runtime || found_profile != profile || application != binding {
        return Err(Denial::BindingMismatch);
    }
    Ok(incarnation)
}

fn take_text(bytes: &[u8]) -> Result<(&str, &[u8]), Denial> {
    let (length, rest) = bytes.split_first_chunk::<4>().ok_or(Denial::Malformed)?;
    let length = u32::from_le_bytes(*length) as usize;
    let text = rest.get(..length).ok_or(Denial::Malformed)?;
    Ok((
        std::str::from_utf8(text).map_err(|_| Denial::Malformed)?,
        &rest[length..],
    ))
}
