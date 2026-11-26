//! Large values stream through fixed windows charged to the in-flight fill
//! ledger; saturation refuses with `MemoryExhausted`, corruption is sticky.
use sha2::{Digest, Sha256};

use super::super::value_reader::{open_value_reader, ValueLocator, ValueReadError};
use super::member_store::{accountant, MemberStore};

const WINDOW: u32 = 4096;
const VALUE_BYTES: usize = 1024 * 1024 + 17;

fn value_member(store: &MemberStore) -> (ValueLocator, [u8; 32]) {
    let prefix = b"field-prefix";
    let value = (0..VALUE_BYTES)
        .map(|index| (index * 31 % 251) as u8)
        .collect::<Vec<_>>();
    let digest: [u8; 32] = Sha256::digest(&value).into();
    let member = store.put_raw([prefix.as_slice(), value.as_slice()].concat());
    (
        ValueLocator {
            member,
            offset: prefix.len() as u64,
            length: VALUE_BYTES as u64,
            value_sha256: digest,
        },
        digest,
    )
}

#[test]
fn value_reader_streams_fixed_windows_and_refuses_saturation() {
    let store = MemberStore::default();
    let (locator, digest) = value_member(&store);
    let accountant = accountant(1 << 20, 2 * u64::from(WINDOW), WINDOW);
    let mut first = open_value_reader(&accountant, &store, locator).unwrap();
    let second = open_value_reader(&accountant, &store, locator).unwrap();
    assert_eq!(first.window_bytes(), WINDOW as usize);
    assert_eq!(
        open_value_reader(&accountant, &store, locator).err(),
        Some(ValueReadError::MemoryExhausted {
            requested: u64::from(WINDOW),
            available: 0
        })
    );
    let mut streamed = Sha256::new();
    let mut windows = 0;
    while let Some(window) = first.next_window().unwrap() {
        assert!(window.len() <= WINDOW as usize);
        streamed.update(window);
        windows += 1;
        assert_eq!(
            accountant.pressure().in_flight_fill_bytes,
            2 * u64::from(WINDOW)
        );
    }
    assert_eq!(<[u8; 32]>::from(streamed.finalize()), digest);
    assert_eq!(windows, VALUE_BYTES.div_ceil(WINDOW as usize));
    drop((first, second));
    assert_eq!(accountant.pressure().in_flight_fill_bytes, 0);
    assert!(open_value_reader(&accountant, &store, locator).is_ok());
}

#[test]
fn value_reader_refuses_corrupt_and_out_of_member_values() {
    let store = MemberStore::default();
    let (locator, _) = value_member(&store);
    let accountant = accountant(1 << 20, 1 << 20, WINDOW);
    let corrupt = ValueLocator {
        value_sha256: [0; 32],
        ..locator
    };
    let mut reader = open_value_reader(&accountant, &store, corrupt).unwrap();
    let outcome = loop {
        match reader.next_window() {
            Ok(Some(_)) => continue,
            other => break other.map(|window| window.is_some()),
        }
    };
    assert_eq!(outcome, Err(ValueReadError::IntegrityMismatch));
    assert_eq!(
        reader.next_window().err(),
        Some(ValueReadError::IntegrityMismatch),
        "sticky"
    );
    let beyond = ValueLocator {
        length: locator.member.length,
        ..locator
    };
    assert_eq!(
        open_value_reader(&accountant, &store, beyond).err(),
        Some(ValueReadError::OutOfBounds)
    );
}
