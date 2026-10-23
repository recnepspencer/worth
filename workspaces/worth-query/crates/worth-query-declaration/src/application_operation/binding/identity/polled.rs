//! Canonical computation digests with original-request polling and fallible allocation.
use super::{
    encode_framed,
    encoder::{HashingSink, PollingAdmission},
    partition,
};
use crate::application_program::{ApplicationComputationInput, ApplicationComputationPartition};
use partition::ApplicationComputationPartitionIdentityDenial as Denial;
use serde::Serialize;
use std::fmt::Debug;

/// Derive the input digest, polling controls throughout canonical encoding.
pub fn application_computation_input_digest_polled<Input, F, E>(
    value: &Input::Value,
    poll: &mut F,
) -> Result<[u8; 32], Denial<E>>
where
    Input: ApplicationComputationInput,
    F: FnMut() -> Result<(), E>,
    E: Debug,
{
    digest(partition::INPUT_VALUE_DOMAIN, Input::IDENTITY, value, poll)
}

/// Derive an item's digest from its complete canonical serialized value.
pub fn application_computation_item_digest_polled<Item, F, E>(
    value: &Item,
    poll: &mut F,
) -> Result<[u8; 32], Denial<E>>
where
    Item: ApplicationComputationPartition,
    F: FnMut() -> Result<(), E>,
    E: Debug,
{
    digest(partition::ITEM_VALUE_DOMAIN, Item::IDENTITY, value, poll)
}

/// Derive a key's digest; the existing partition identity derivation consumes it.
pub fn application_computation_partition_digest_polled<Key, F, E>(
    value: &Key,
    poll: &mut F,
) -> Result<[u8; 32], Denial<E>>
where
    Key: ApplicationComputationPartition,
    F: FnMut() -> Result<(), E>,
    E: Debug,
{
    digest(partition::PARTITION_KEY_DOMAIN, Key::IDENTITY, value, poll)
}

fn digest<T, F, E>(
    domain: &str,
    identity: &'static str,
    value: &T,
    poll: &mut F,
) -> Result<[u8; 32], Denial<E>>
where
    T: Serialize + ?Sized,
    F: FnMut() -> Result<(), E>,
    E: Debug,
{
    let mut sink = HashingSink::with_admission(PollingAdmission::new(poll));
    let encoded = encode_framed(&mut sink, domain, identity, value);
    if let Some(error) = sink.take_terminal_failure() {
        return Err(partition::identity_denial(identity, error));
    }
    encoded.map_err(|error| partition::identity_denial(identity, error))?;
    sink.finish()
        .map_err(|error| partition::identity_denial(identity, error))
}
