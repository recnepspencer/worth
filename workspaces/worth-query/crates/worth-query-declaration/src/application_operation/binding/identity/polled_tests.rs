//! Polling and accounted controls consume one canonical grammar and preserve stops.
use super::*;
use crate::application_program::{ApplicationComputationInput, ApplicationComputationPartition};
use serde::{
    ser::{SerializeMap, SerializeSeq},
    Serialize, Serializer,
};
use std::{
    convert::Infallible,
    sync::{
        atomic::{AtomicBool, Ordering},
        Arc,
    },
};

struct Map(Vec<(&'static str, Vec<u64>)>);
impl Serialize for Map {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        let mut map = serializer.serialize_map(Some(self.0.len()))?;
        for (key, value) in &self.0 {
            map.serialize_entry(key, value)?;
        }
        map.end()
    }
}
impl ApplicationComputationPartition for Map {
    const IDENTITY: &'static str = "polled-map.v1";
}
struct Input;
impl ApplicationComputationInput for Input {
    type Value = Map;
    const IDENTITY: &'static str = "polled-input.v1";
}

#[test]
fn polled_input_item_and_key_keep_accounted_identities_and_map_order() {
    let entries = vec![
        ("z", (0..128).collect()),
        ("a", vec![9, 3]),
        ("middle", vec![]),
    ];
    let forward = Map(entries.clone());
    let reverse = Map(entries.into_iter().rev().collect());
    let mut account = |_| Ok::<_, Infallible>(());
    let mut poll = || Ok::<_, Infallible>(());
    let key = application_computation_partition_identity(&forward, &mut account).unwrap();
    assert_eq!(
        application_computation_partition_digest_polled(&forward, &mut poll).unwrap(),
        *key.digest()
    );
    assert_eq!(
        application_computation_partition_digest_polled(&reverse, &mut poll).unwrap(),
        *key.digest()
    );
    let item = application_computation_item_digest(&forward, &mut account).unwrap();
    assert_eq!(
        application_computation_item_digest_polled(&reverse, &mut poll).unwrap(),
        item
    );
    let input =
        application_computation_input_digest::<Input, _, _>(&forward, &mut account).unwrap();
    assert_eq!(
        application_computation_input_digest_polled::<Input, _, _>(&reverse, &mut poll).unwrap(),
        input
    );
    assert_ne!(input, item, "input and item domains remain distinct");
    assert_ne!(item, *key.digest(), "item and key domains remain distinct");
}

struct Swallows(Arc<AtomicBool>);
impl Serialize for Swallows {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        let mut sequence = serializer.serialize_seq(None)?;
        self.0.store(true, Ordering::SeqCst);
        let _ = sequence.serialize_element(&17_u64);
        let _ = sequence.serialize_element(&18_u64);
        sequence.end()
    }
}
impl ApplicationComputationPartition for Swallows {
    const IDENTITY: &'static str = "swallows-stop.v1";
}

#[derive(Debug, PartialEq)]
enum Stop {
    OriginalCancellation,
}

#[test]
fn polled_encoding_retains_the_original_stop_when_serialization_catches_it() {
    let stopped = Arc::new(AtomicBool::new(false));
    let value = Swallows(stopped.clone());
    let result = application_computation_item_digest_polled(&value, &mut || {
        if stopped.load(Ordering::SeqCst) {
            Err(Stop::OriginalCancellation)
        } else {
            Ok(())
        }
    });
    assert!(matches!(
        result,
        Err(ApplicationComputationPartitionIdentityDenial::Admission(
            Stop::OriginalCancellation
        ))
    ));
}

#[test]
fn polled_encoding_preserves_serialization_rejection() {
    struct Rejected;
    impl Serialize for Rejected {
        fn serialize<S: Serializer>(&self, _: S) -> Result<S::Ok, S::Error> {
            Err(serde::ser::Error::custom("original invalid value"))
        }
    }
    impl ApplicationComputationPartition for Rejected {
        const IDENTITY: &'static str = "rejected.v1";
    }
    assert!(matches!(
        application_computation_item_digest_polled(&Rejected, &mut || Ok::<_, Infallible>(())),
        Err(ApplicationComputationPartitionIdentityDenial::Key(_))
    ));
}
