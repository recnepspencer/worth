//! The home descriptor names the Relational runtime a home belongs to. It is
//! published once, when the home is created, under the anchor
//! `relational.home`, and checked on every open before any history is read.
//!
//! Payload (little-endian): magic `WSRHOME1`, format version `u16`, then the
//! runtime name and the profile name, each as a `u32` length and UTF-8 bytes.

use worth_relational::facade::runtime::RelationalRuntime;
use worth_store::physical_runtime::{
    AnchoredRecordName, AnchoredRecordPublication, AnchoredRecordPublicationFailure,
    AnchoredRecordReadFailure, AnchoredRecordsDenial, PhysicalMutationDeadline,
    ServingPhysicalRuntime,
};

const ANCHOR: &str = "relational.home";
const MAGIC: &[u8; 8] = b"WSRHOME1";
const FORMAT_VERSION: u16 = 1;
const MAXIMUM_DESCRIPTOR_BYTES: u64 = 64 << 10;
const PUBLICATION_DEADLINE_MILLIS: u64 = 30_000;

/// Why the home descriptor could not be read or written.
#[derive(Debug)]
#[non_exhaustive]
pub enum RelationalStoreHomeDescriptorRefusal {
    Unavailable(AnchoredRecordsDenial),
    Read(AnchoredRecordReadFailure),
    /// The descriptor record is intact for Store but is not a descriptor this
    /// build can read.
    Malformed,
    Publication(AnchoredRecordPublicationFailure),
}

type Refusal = RelationalStoreHomeDescriptorRefusal;

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct HomeDescriptor {
    runtime_name: String,
    profile: String,
}

impl HomeDescriptor {
    pub(crate) fn of(runtime: &RelationalRuntime) -> Self {
        let config = runtime.config();
        Self {
            runtime_name: config.execution.runtime_name.clone(),
            profile: format!("{:?}", config.profile),
        }
    }

    pub(crate) fn describe(&self) -> String {
        format!(
            "runtime {:?} with profile {}",
            self.runtime_name, self.profile
        )
    }

    fn encode(&self) -> Vec<u8> {
        let mut bytes = Vec::with_capacity(18 + self.runtime_name.len() + self.profile.len());
        bytes.extend_from_slice(MAGIC);
        bytes.extend_from_slice(&FORMAT_VERSION.to_le_bytes());
        for text in [&self.runtime_name, &self.profile] {
            let length = u32::try_from(text.len()).expect("descriptor text fits a u32 length");
            bytes.extend_from_slice(&length.to_le_bytes());
            bytes.extend_from_slice(text.as_bytes());
        }
        bytes
    }

    fn decode(bytes: &[u8]) -> Option<Self> {
        let rest = bytes.strip_prefix(MAGIC)?;
        let (version, rest) = rest.split_first_chunk::<2>()?;
        if u16::from_le_bytes(*version) != FORMAT_VERSION {
            return None;
        }
        let (runtime_name, rest) = decode_text(rest)?;
        let (profile, rest) = decode_text(rest)?;
        rest.is_empty().then_some(Self {
            runtime_name,
            profile,
        })
    }
}

fn decode_text(bytes: &[u8]) -> Option<(String, &[u8])> {
    let (length, rest) = bytes.split_first_chunk::<4>()?;
    let length = usize::try_from(u32::from_le_bytes(*length)).ok()?;
    let text = rest.get(..length)?;
    Some((String::from_utf8(text.to_vec()).ok()?, &rest[length..]))
}

fn anchor() -> AnchoredRecordName {
    AnchoredRecordName::new(ANCHOR).expect("the home descriptor anchor name is admissible")
}

/// The home's descriptor, or `None` when the home has none yet.
pub(crate) fn read(serving: &ServingPhysicalRuntime) -> Result<Option<HomeDescriptor>, Refusal> {
    let records = serving.anchored_records().map_err(Refusal::Unavailable)?;
    let Some(record) = records
        .read(anchor(), MAXIMUM_DESCRIPTOR_BYTES)
        .map_err(Refusal::Read)?
    else {
        return Ok(None);
    };
    HomeDescriptor::decode(record.bytes())
        .map(Some)
        .ok_or(Refusal::Malformed)
}

/// Publish the first descriptor of a new home.
pub(crate) fn publish(
    serving: &ServingPhysicalRuntime,
    descriptor: &HomeDescriptor,
) -> Result<(), Refusal> {
    let records = serving.anchored_records().map_err(Refusal::Unavailable)?;
    let deadline = PhysicalMutationDeadline::after_milliseconds(PUBLICATION_DEADLINE_MILLIS)
        .expect("the descriptor deadline is representable");
    records
        .publish(AnchoredRecordPublication::new(
            anchor(),
            0,
            descriptor.encode(),
            crate::physical_store::placement(),
            deadline,
        ))
        .map(|_| ())
        .map_err(Refusal::Publication)
}
