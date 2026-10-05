use serde::Serialize;

use super::super::{
    application_computation_partition_identity, canonical_identity,
    ApplicationComputationPartitionIdentity, ApplicationComputationPartitionIdentityDenial,
    CanonicalEncodingCharge,
};
use crate::application_program::ApplicationComputationPartition;

#[derive(Serialize)]
struct Region(u32);

impl ApplicationComputationPartition for Region {
    const IDENTITY: &'static str = "test.partition-identity.region.v1";
}

#[derive(Serialize)]
#[serde(rename = "Region")]
struct OtherRegion(u32);

impl ApplicationComputationPartition for OtherRegion {
    const IDENTITY: &'static str = "test.partition-identity.other-region.v1";
}

struct Refusing;

impl Serialize for Refusing {
    fn serialize<S: serde::Serializer>(&self, _: S) -> Result<S::Ok, S::Error> {
        Err(serde::ser::Error::custom("refused"))
    }
}

impl ApplicationComputationPartition for Refusing {
    const IDENTITY: &'static str = "test.partition-identity.refusing.v1";
}

fn derive<Key: ApplicationComputationPartition>(
    key: &Key,
) -> ApplicationComputationPartitionIdentity {
    application_computation_partition_identity(key, &mut |_| Ok::<_, ()>(()))
        .expect("the key encodes")
}

#[test]
fn the_digest_is_the_canonical_encoding_under_the_key_domain_and_type() {
    let derived = derive(&Region(7));
    let expected = canonical_identity(
        "worth-query.computation-partition-key.v1",
        Region::IDENTITY,
        &Region(7),
    )
    .expect("canonical encoding");
    assert_eq!(*derived.digest(), expected.identity());
    assert_eq!(derived.work(), expected.work());
    assert_eq!(derived, derive(&Region(7)));
    assert_ne!(derived.digest(), derive(&Region(8)).digest());
    assert_ne!(derived.digest(), derive(&OtherRegion(7)).digest());
}

#[test]
fn the_partition_identity_is_the_first_eight_digest_bytes_big_endian() {
    let derived = derive(&Region(7));
    let mut prefix = [0_u8; 8];
    prefix.copy_from_slice(&derived.digest()[..8]);
    assert_eq!(derived.partition().value(), u64::from_be_bytes(prefix));

    let mut left = [0_u8; 32];
    left[..8].copy_from_slice(&[1, 2, 3, 4, 5, 6, 7, 8]);
    let mut right = left;
    right[31] = 9;
    assert_eq!(
        ApplicationComputationPartitionIdentity::partition_of(&left).value(),
        0x0102_0304_0506_0708
    );
    assert_eq!(
        ApplicationComputationPartitionIdentity::partition_of(&left),
        ApplicationComputationPartitionIdentity::partition_of(&right)
    );
}

/// The encoder and the truncation are frozen: a canonical result reduces in
/// this order, so a changed vector is a changed result meaning.
#[test]
fn the_derivation_is_frozen() {
    assert_eq!(derive(&Region(7)).partition().value(), FROZEN_REGION_SEVEN);
}

// SHA-256 over the big-endian length and bytes of the domain, then of the key
// type's identity, then the newtype tag, the name `Region` and unsigned 7.
const FROZEN_REGION_SEVEN: u64 = 0xc616_3308_4b9f_e7d7;

#[test]
fn every_charge_is_requested_and_a_refused_charge_is_the_denial() {
    let mut work = 0_u64;
    let derived = application_computation_partition_identity(&Region(7), &mut |charge| {
        if let CanonicalEncodingCharge::Work(units) = charge {
            work += units;
        }
        Ok::<_, ()>(())
    })
    .expect("the key encodes");
    assert!(work > 0);
    assert_eq!(derived, derive(&Region(7)));

    let refused = application_computation_partition_identity(&Region(7), &mut |_| Err("no budget"));
    assert!(matches!(
        refused,
        Err(ApplicationComputationPartitionIdentityDenial::Admission(
            "no budget"
        ))
    ));
}

#[test]
fn a_key_that_refuses_to_serialize_has_no_identity() {
    let refused = application_computation_partition_identity(&Refusing, &mut |_| Ok::<_, ()>(()));
    assert!(matches!(
        refused,
        Err(ApplicationComputationPartitionIdentityDenial::Key(_))
    ));
}
