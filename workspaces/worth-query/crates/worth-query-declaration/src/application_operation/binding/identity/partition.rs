//! The one derivation of a computation partition's identity.
//!
//! A partition key has no notion of sameness but its canonical encoding. Its
//! digest is the SHA-256 of that encoding under the partition-key domain and
//! the key type's declared identity, and its partition identity is the first
//! eight bytes of the digest, big-endian. Partitions reduce in identity order,
//! so the encoder and the truncation are part of a canonical result's meaning.
//!
//! A partitioned computation's input value has the same canonical encoding,
//! under its own domain and the input's declared identity: its digest is what
//! makes two input values the same input. An item a computation partitions is
//! encoded the same way under the item domain: its digest is what makes two
//! values of one item the same value.

use std::fmt::Debug;

use serde::Serialize;

use worth_foundational::facade::PartitionIdentity;

use super::{
    canonical_identity_admitted, encoder::CanonicalEncodeError, rejected, ApplicationCanonicalWork,
    CanonicalEncodingCharge,
};
use crate::application_program::{ApplicationComputationInput, ApplicationComputationPartition};
use crate::application_schema::ApplicationValueEncodeDenial;
use crate::portable_identity::WorthQueryPortableTypeIdentity;

const PARTITION_KEY_DOMAIN: &str = "worth-query.computation-partition-key.v1";
const INPUT_VALUE_DOMAIN: &str = "worth-query.computation-input-value.v1";
const ITEM_VALUE_DOMAIN: &str = "worth-query.computation-item-value.v1";

/// One partition key's digest, its partition identity and the work that
/// derived them.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ApplicationComputationPartitionIdentity {
    digest: [u8; 32],
    work: ApplicationCanonicalWork,
}

impl ApplicationComputationPartitionIdentity {
    /// Names the partition a digest belongs to: its first eight bytes,
    /// big-endian. Two different digests can share one partition identity.
    pub const fn partition_of(digest: &[u8; 32]) -> PartitionIdentity {
        let [b0, b1, b2, b3, b4, b5, b6, b7, ..] = *digest;
        PartitionIdentity::new(u64::from_be_bytes([b0, b1, b2, b3, b4, b5, b6, b7]))
    }

    /// The SHA-256 of the key's canonical encoding: what makes two keys the
    /// same key.
    pub const fn digest(&self) -> &[u8; 32] {
        &self.digest
    }

    /// The partition the key names.
    pub const fn partition(&self) -> PartitionIdentity {
        Self::partition_of(&self.digest)
    }

    /// The deterministic work that derived the digest.
    pub const fn work(&self) -> ApplicationCanonicalWork {
        self.work
    }
}

/// Why a partition key or an input value has no identity.
#[derive(Debug)]
pub enum ApplicationComputationPartitionIdentityDenial<E: Debug> {
    /// The value's `Serialize` form refused to encode.
    Key(ApplicationValueEncodeDenial),
    /// The caller's admission refused a work or scratch charge.
    Admission(E),
    CapacityOverflow,
    Allocation,
}

/// Derives a partition key's identity, requesting every work and scratch
/// charge from `admission` before the hash, copy or growth it pays for.
pub fn application_computation_partition_identity<Key, F, E>(
    key: &Key,
    admission: &mut F,
) -> Result<ApplicationComputationPartitionIdentity, ApplicationComputationPartitionIdentityDenial<E>>
where
    Key: ApplicationComputationPartition,
    F: FnMut(CanonicalEncodingCharge) -> Result<(), E>,
    E: Debug,
{
    admitted(PARTITION_KEY_DOMAIN, Key::IDENTITY, key, admission).map(|identity| {
        ApplicationComputationPartitionIdentity {
            digest: identity.identity(),
            work: identity.work(),
        }
    })
}

/// Derives the digest of a partitioned computation's input value, requesting
/// every work and scratch charge from `admission` as a partition key does.
pub fn application_computation_input_digest<Input, F, E>(
    value: &Input::Value,
    admission: &mut F,
) -> Result<[u8; 32], ApplicationComputationPartitionIdentityDenial<E>>
where
    Input: ApplicationComputationInput,
    F: FnMut(CanonicalEncodingCharge) -> Result<(), E>,
    E: Debug,
{
    admitted(INPUT_VALUE_DOMAIN, Input::IDENTITY, value, admission)
        .map(|identity| identity.identity())
}

/// Derives the digest of one item a partitioned computation partitions,
/// requesting every work and scratch charge from `admission` as a partition
/// key does. Two values of one item are the same value exactly when their
/// digests are equal.
pub fn application_computation_item_digest<Item, F, E>(
    item: &Item,
    admission: &mut F,
) -> Result<[u8; 32], ApplicationComputationPartitionIdentityDenial<E>>
where
    Item: ApplicationComputationPartition,
    F: FnMut(CanonicalEncodingCharge) -> Result<(), E>,
    E: Debug,
{
    admitted(ITEM_VALUE_DOMAIN, Item::IDENTITY, item, admission).map(|identity| identity.identity())
}

fn admitted<T, F, E>(
    domain: &str,
    identity: &'static str,
    value: &T,
    admission: &mut F,
) -> Result<super::ApplicationCanonicalIdentity, ApplicationComputationPartitionIdentityDenial<E>>
where
    T: Serialize + ?Sized,
    F: FnMut(CanonicalEncodingCharge) -> Result<(), E>,
    E: Debug,
{
    match canonical_identity_admitted(domain, identity, value, admission) {
        Ok(identity) => Ok(identity),
        Err(CanonicalEncodeError::Serialization) => {
            Err(ApplicationComputationPartitionIdentityDenial::Key(
                rejected(WorthQueryPortableTypeIdentity::declared(identity)),
            ))
        }
        Err(CanonicalEncodeError::Admission(error)) => Err(
            ApplicationComputationPartitionIdentityDenial::Admission(error),
        ),
        Err(CanonicalEncodeError::AdmissionDeferred) => {
            unreachable!("the canonical owner returns its latched admission cause")
        }
        Err(CanonicalEncodeError::CapacityOverflow) => {
            Err(ApplicationComputationPartitionIdentityDenial::CapacityOverflow)
        }
        Err(CanonicalEncodeError::Allocation) => {
            Err(ApplicationComputationPartitionIdentityDenial::Allocation)
        }
    }
}
