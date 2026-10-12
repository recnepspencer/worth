//! Declared nested checkpoints and canonical partition metadata, not a run.
use super::super::structural_cost::Operations;
use super::Partition;

fn identity(key: u64) -> u64 {
    use sha2::{Digest, Sha256};
    // Canonical partition-key contract: domain/identity BE lengths, then
    // the Key newtype (tag 14) and minimal unsigned LEB128 (tag 3).
    let mut bytes = Vec::new();
    for text in [
        "worth-query.computation-partition-key.v1",
        "matrix-partition",
    ] {
        bytes.extend_from_slice(&u64::try_from(text.len()).unwrap().to_be_bytes());
        bytes.extend_from_slice(text.as_bytes());
    }
    bytes.extend_from_slice(&[14, 3]);
    bytes.extend_from_slice(b"Key");
    bytes.push(3);
    let mut left = key;
    loop {
        let group = u8::try_from(left & 127).unwrap();
        left >>= 7;
        bytes.push(group | if left == 0 { 0 } else { 128 });
        if left == 0 {
            break;
        }
    }
    let digest = Sha256::digest(bytes);
    u64::from_be_bytes(digest[..8].try_into().unwrap())
}

pub(super) fn checkpoints(parts: &[Partition], failed: bool) -> (u64, u64) {
    let mut ordered = parts.iter().collect::<Vec<_>>();
    ordered.sort_by_key(|part| identity(part.key));
    let kept = if failed {
        ordered.iter().position(|part| part.key == 0).unwrap() + 1
    } else {
        ordered.len()
    };
    // An inner map denial accepts its canonical checkpoint prefix. The outer
    // advancement still pays every member of its dispatched wave. The price
    // twin asserts this prefix in the whole advancement's exact bill even when
    // a compute refusal prevents the completed-result meter from running.
    let cost = Operations::Parallel(
        ordered[..kept]
            .iter()
            .map(|part| Operations::Units(part.work))
            .collect(),
    )
    .cost();
    (cost.work, cost.span)
}
