use std::{cell::Cell, collections::BTreeMap};

use serde::{ser::SerializeMap, Serialize, Serializer};

use super::super::{
    canonical_identity, canonical_identity_admitted,
    encoder::{self, CanonicalEncodeError},
    CanonicalEncodingCharge,
};

struct DuplicateKeyMap {
    reverse: bool,
}

impl Serialize for DuplicateKeyMap {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        let mut map = serializer.serialize_map(Some(2))?;
        let first = BTreeMap::from([("nested", vec![7_u8, 8, 9])]);
        let second = BTreeMap::from([("nested", vec![1_u8, 2, 3])]);
        if self.reverse {
            map.serialize_entry(&1_u64, &second)?;
            map.serialize_entry(&1_u8, &first)?;
        } else {
            map.serialize_entry(&1_u8, &first)?;
            map.serialize_entry(&1_u64, &second)?;
        }
        map.end()
    }
}

#[test]
fn admitted_nested_duplicate_key_map_preserves_bytes_digest_and_work() {
    let forward = DuplicateKeyMap { reverse: false };
    let reverse = DuplicateKeyMap { reverse: true };
    let mut forward_bytes = Vec::new();
    let mut reverse_bytes = Vec::new();
    encoder::encode_into(&mut forward_bytes, &forward).unwrap();
    encoder::encode_into(&mut reverse_bytes, &reverse).unwrap();
    assert_eq!(forward_bytes, reverse_bytes);
    let hex = |bytes: &[u8]| {
        bytes
            .iter()
            .map(|byte| format!("{byte:02x}"))
            .collect::<String>()
    };
    // Independent v1 wire contract: map tag, sorted full (key,value) pairs,
    // nested map/sequence framing, then END. Both keys encode as unsigned 1.
    assert_eq!(
        hex(&forward_bytes),
        "14010301140107066e6573746564100103010103020103030000\
         010301140107066e657374656410010307010308010309000000"
    );

    let plain = canonical_identity("test", "scope", &forward).unwrap();
    let mut charges = Vec::new();
    let admitted = canonical_identity_admitted("test", "scope", &reverse, &mut |charge| {
        charges.push(charge);
        Ok::<_, ()>(())
    })
    .unwrap();
    assert_eq!(plain, admitted);
    assert_eq!(
        hex(&plain.identity()),
        "e1b600d34e1c117487130fedfa7ab64a352faf1be01db0913bb3ada39ec159db"
    );
    assert!(charges
        .iter()
        .any(|charge| matches!(charge, CanonicalEncodingCharge::Scratch(_))));
    assert!(charges
        .iter()
        .any(|charge| matches!(charge, CanonicalEncodingCharge::Work(_))));
}

struct SwallowsWork<'a>(&'a Cell<bool>);

impl Serialize for SwallowsWork<'_> {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        use serde::ser::SerializeSeq;
        let mut seq = serializer.serialize_seq(Some(2))?;
        let _ = seq.serialize_element(&"x".repeat(100));
        self.0.set(true);
        let _ = seq.serialize_element(&"later");
        seq.end()
    }
}

struct LaterKey<'a>(&'a Cell<bool>);

impl Serialize for LaterKey<'_> {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        self.0.set(true);
        serializer.serialize_str("later")
    }
}

struct SwallowsScratch<'a>(&'a Cell<bool>, &'a Cell<bool>);

impl Serialize for SwallowsScratch<'_> {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        let mut map = serializer.serialize_map(Some(1))?;
        let _ = map.serialize_key(&"key");
        self.0.set(true);
        let _ = map.serialize_key(&LaterKey(self.1));
        map.end()
    }
}

#[test]
fn swallowed_work_and_scratch_refusals_remain_typed_and_stop_later_charges() {
    let later = Cell::new(false);
    let mut refused = false;
    let mut later_callback = false;
    let denial =
        canonical_identity_admitted("test", "scope", &SwallowsWork(&later), &mut |charge| {
            if refused {
                later_callback = true;
            }
            if matches!(charge, CanonicalEncodingCharge::Work(units) if units > 10) {
                refused = true;
                return Err("work");
            }
            Ok(())
        });
    assert!(
        later.get(),
        "custom Serialize caught the first write refusal"
    );
    assert!(matches!(
        denial,
        Err(CanonicalEncodeError::Admission("work"))
    ));
    assert!(!later_callback, "the latched sink never reenters admission");

    later.set(false);
    let later_key_entered = Cell::new(false);
    refused = false;
    later_callback = false;
    let denial = canonical_identity_admitted(
        "test",
        "scope",
        &SwallowsScratch(&later, &later_key_entered),
        &mut |charge| {
            if refused {
                later_callback = true;
            }
            if matches!(charge, CanonicalEncodingCharge::Scratch(_)) {
                refused = true;
                return Err("scratch");
            }
            Ok(())
        },
    );
    assert!(
        later.get(),
        "custom Serialize caught the key-buffer refusal"
    );
    assert!(
        !later_key_entered.get(),
        "latched admission prevents a later key serializer"
    );
    assert!(matches!(
        denial,
        Err(CanonicalEncodeError::Admission("scratch"))
    ));
    assert!(!later_callback);
}

#[test]
fn terminal_capacity_and_allocation_errors_block_later_custom_serialization() {
    use encoder::Sink;
    let entered = Cell::new(false);
    let mut capacity = encoder::HashingSink::new();
    let _ = capacity.fail_capacity();
    assert!(matches!(
        encoder::encode_into(&mut capacity, &LaterKey(&entered)),
        Err(CanonicalEncodeError::CapacityOverflow)
    ));
    assert!(!entered.get());
    assert!(matches!(
        capacity.finish(),
        Err(CanonicalEncodeError::CapacityOverflow)
    ));

    let mut allocation = encoder::HashingSink::new();
    let _ = allocation.fail_allocation();
    assert!(matches!(
        encoder::encode_into(&mut allocation, &LaterKey(&entered)),
        Err(CanonicalEncodeError::Allocation)
    ));
    assert!(!entered.get());
    assert!(matches!(
        allocation.finish(),
        Err(CanonicalEncodeError::Allocation)
    ));
}

#[test]
fn nested_map_work_and_scratch_deny_before_later_serialization() {
    let value = DuplicateKeyMap { reverse: false };
    let mut work = 0_u64;
    let mut scratch = 0_u64;
    canonical_identity_admitted("test", "scope", &value, &mut |charge| {
        match charge {
            CanonicalEncodingCharge::Work(units) => work += units,
            CanonicalEncodingCharge::Scratch(bytes) => scratch += bytes,
        }
        Ok::<_, ()>(())
    })
    .unwrap();
    assert!(work > 0 && scratch > 0);

    let visited = Cell::new(0_u64);
    let mut remaining = work - 1;
    let refusal = canonical_identity_admitted("test", "scope", &value, &mut |charge| {
        visited.set(visited.get() + 1);
        if let CanonicalEncodingCharge::Work(units) = charge {
            if units > remaining {
                return Err("work");
            }
            remaining -= units;
        }
        Ok(())
    });
    assert!(matches!(
        refusal,
        Err(CanonicalEncodeError::Admission("work"))
    ));
    assert!(visited.get() > 0);

    let mut remaining = scratch - 1;
    let refusal = canonical_identity_admitted("test", "scope", &value, &mut |charge| {
        if let CanonicalEncodingCharge::Scratch(bytes) = charge {
            if bytes > remaining {
                return Err("scratch");
            }
            remaining -= bytes;
        }
        Ok(())
    });
    assert!(matches!(
        refusal,
        Err(CanonicalEncodeError::Admission("scratch"))
    ));
}

#[test]
fn two_derivations_share_one_cumulative_admission() {
    let mut first_work = 0_u64;
    let mut second_work = 0_u64;
    canonical_identity_admitted("key", "scope", &23_u32, &mut |charge| {
        if let CanonicalEncodingCharge::Work(units) = charge {
            first_work += units;
        }
        Ok::<_, ()>(())
    })
    .unwrap();
    canonical_identity_admitted("input", "scope", &"payload", &mut |charge| {
        if let CanonicalEncodingCharge::Work(units) = charge {
            second_work += units;
        }
        Ok::<_, ()>(())
    })
    .unwrap();
    let mut remaining = first_work + second_work - 1;
    canonical_identity_admitted("key", "scope", &23_u32, &mut |charge| {
        if let CanonicalEncodingCharge::Work(units) = charge {
            if units > remaining {
                return Err("work");
            }
            remaining -= units;
        }
        Ok(())
    })
    .unwrap();
    let refusal = canonical_identity_admitted("input", "scope", &"payload", &mut |charge| {
        if let CanonicalEncodingCharge::Work(units) = charge {
            if units > remaining {
                return Err("work");
            }
            remaining -= units;
        }
        Ok(())
    });
    assert!(matches!(
        refusal,
        Err(CanonicalEncodeError::Admission("work"))
    ));
}
