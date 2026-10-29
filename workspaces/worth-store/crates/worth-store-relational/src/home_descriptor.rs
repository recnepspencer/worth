//! The home descriptor names the Relational runtime a home belongs to. It is
//! published under `home` before service on every open. Store identity is
//! the home identity: one home owns exactly one Store namespace.
//!
//! Payload (little-endian): magic `WSRHOME1`, format version `u16`, then the
//! Store-issued identity (16 bytes), incarnation `u64`, runtime and profile
//! as `u32` lengths and UTF-8, then a `u16` tagged-binding count. Binding entries
//! are tag `u16`, byte length `u32`, opaque bytes. Phase 2 emits no bindings;
//! later phases add tags without changing this format. Unknown tags refuse.

use worth_relational::facade::runtime::RelationalRuntime;
use worth_store::physical_runtime::{
    AnchoredRecordName, AnchoredRecordPublication, AnchoredRecordPublicationFailure,
    AnchoredRecordReadFailure, AnchoredRecordsDenial, PhysicalMutationDeadline,
    ServingPhysicalRuntime,
};

const ANCHOR: &str = "home";
const MAGIC: &[u8; 8] = b"WSRHOME1";
const FORMAT_VERSION: u16 = 2;
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
    UnsupportedDescriptorVersion {
        observed: u16,
    },
    UnsupportedBinding {
        tag: u16,
    },
    IncarnationExhausted,
    IdentityMismatch,

    Publication(AnchoredRecordPublicationFailure),
}

type Refusal = RelationalStoreHomeDescriptorRefusal;

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct HomeDescriptor {
    pub(crate) identity: crate::StableStoreIdentity,
    pub(crate) incarnation: u64,
    generation: u64,
    runtime_name: String,
    profile: String,
}

impl HomeDescriptor {
    pub(crate) fn of(runtime: &RelationalRuntime, identity: crate::StableStoreIdentity) -> Self {
        let config = runtime.config();
        Self {
            identity,
            incarnation: 0,
            generation: 0,
            runtime_name: config.execution.runtime_name.clone(),
            profile: format!("{:?}", config.profile),
        }
    }

    pub(crate) fn same_binding(&self, other: &Self) -> bool {
        self.identity == other.identity
            && self.runtime_name == other.runtime_name
            && self.profile == other.profile
    }

    pub(crate) fn advanced(mut self) -> Result<Self, Refusal> {
        self.incarnation = self
            .incarnation
            .checked_add(1)
            .ok_or(Refusal::IncarnationExhausted)?;
        Ok(self)
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
        bytes.extend_from_slice(&self.identity.bytes());
        bytes.extend_from_slice(&self.incarnation.to_le_bytes());
        for text in [&self.runtime_name, &self.profile] {
            let length = u32::try_from(text.len()).expect("descriptor text fits a u32 length");
            bytes.extend_from_slice(&length.to_le_bytes());
            bytes.extend_from_slice(text.as_bytes());
        }
        bytes.extend_from_slice(&0_u16.to_le_bytes());
        bytes
    }

    fn decode(bytes: &[u8], identity: crate::StableStoreIdentity) -> Result<Self, Refusal> {
        let rest = bytes.strip_prefix(MAGIC).ok_or(Refusal::Malformed)?;
        let (version, rest) = rest.split_first_chunk::<2>().ok_or(Refusal::Malformed)?;
        let observed = u16::from_le_bytes(*version);
        if observed != FORMAT_VERSION {
            return Err(Refusal::UnsupportedDescriptorVersion { observed });
        }
        let (recorded_identity, rest) = rest.split_first_chunk::<16>().ok_or(Refusal::Malformed)?;
        if *recorded_identity != identity.bytes() {
            return Err(Refusal::IdentityMismatch);
        }
        let (incarnation, rest) = rest.split_first_chunk::<8>().ok_or(Refusal::Malformed)?;
        let incarnation = u64::from_le_bytes(*incarnation);
        if incarnation == 0 {
            return Err(Refusal::Malformed);
        }
        let (runtime_name, rest) = decode_text(rest).ok_or(Refusal::Malformed)?;
        let (profile, rest) = decode_text(rest).ok_or(Refusal::Malformed)?;
        let (count, rest) = rest.split_first_chunk::<2>().ok_or(Refusal::Malformed)?;
        if u16::from_le_bytes(*count) != 0 {
            let (tag, _) = rest.split_first_chunk::<2>().ok_or(Refusal::Malformed)?;
            return Err(Refusal::UnsupportedBinding {
                tag: u16::from_le_bytes(*tag),
            });
        }
        if !rest.is_empty() {
            return Err(Refusal::Malformed);
        }
        Ok(Self {
            identity,
            incarnation,
            generation: 0,
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
    if let Some(legacy) = records
        .read(
            AnchoredRecordName::new("relational.home").unwrap(),
            MAXIMUM_DESCRIPTOR_BYTES,
        )
        .map_err(Refusal::Read)?
    {
        let observed = legacy
            .bytes()
            .get(8..10)
            .and_then(|b| b.try_into().ok())
            .map(u16::from_le_bytes)
            .unwrap_or(0);
        return Err(Refusal::UnsupportedDescriptorVersion { observed });
    }
    let Some(record) = records
        .read(anchor(), MAXIMUM_DESCRIPTOR_BYTES)
        .map_err(Refusal::Read)?
    else {
        return Ok(None);
    };
    let mut descriptor = HomeDescriptor::decode(record.bytes(), serving.store_identity())?;
    descriptor.generation = record.version().generation();
    if descriptor.identity != serving.store_identity() {
        return Err(Refusal::IdentityMismatch);
    }
    Ok(Some(descriptor))
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
            descriptor.generation,
            descriptor.encode(),
            crate::physical_store::placement(),
            deadline,
        ))
        .map(|_| ())
        .map_err(Refusal::Publication)
}
