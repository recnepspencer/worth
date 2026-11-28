//! proof-mirror: crates/worth-runtime-world/src/identity/branch.rs:70-78 sha256:f02158d153d3616c474c9dde6bc5491318ca335f72ced1a2c041037b831b95b2
//! proof-mirror: crates/worth-runtime-world/src/durability/artifact/creation.rs:85-86 sha256:6d3e6f1906d6d42cb768c074b30d293a2216348c8f183eb1a91f668bada8473e
//! proof-mirror: crates/worth-runtime-world/src/branch/reference_cell/successor_validation.rs:51-57 sha256:03e8b6a881ca1e02cfb0c8edeabf861d6ba425259da4908d5344696c429e1949
//! proof-mirror: crates/worth-runtime-world/src/durability/readmission/open.rs:16-38 sha256:b776adbb2aa1aa8ab01f8c66dfdd0951561627c75f16848909aea840725a54ff
//! proof-mirror: crates/worth-runtime-world/src/durability/readmission/journal.rs:251-255 sha256:a8b8cff27e902f2b09f1605090a7d3830d881465db528b16662b98680a1fdaac
//!
//! World reference and recovery transitions after the owners supply the exact
//! selected commit, its direct parent, and the admitted journal disposition.
use vstd::prelude::*;

verus! {

fn fork_generation(source: u64) -> (destination: Option<u64>)
    ensures
        destination.is_none() <==> source == u64::MAX,
        destination.is_some() ==> destination.unwrap() as int == source as int + 1,
{
    source.checked_add(1)
}

fn fast_forward(current: u64, first_parent: Option<u64>) -> (admitted: bool)
    ensures admitted <==> first_parent == Some(current),
{
    match first_parent {
        Some(parent) => parent == current,
        None => false,
    }
}

enum Disposition {
    Open,
    Continued,
    Released,
}

fn reclaim_expected(
    published: bool,
    disposition: Disposition,
    expected: u64,
) -> (reclaim: Option<u64>)
    ensures
        reclaim.is_none() <==> published || disposition == Disposition::Released,
        reclaim.is_some() ==> reclaim.unwrap() == expected,
{
    match (published, disposition) {
        (true, _) | (_, Disposition::Released) => None,
        (false, _) => Some(expected),
    }
}

fn record_release(disposition: Disposition) -> (next: Option<Disposition>)
    ensures
        next.is_some() <==> disposition == Disposition::Open,
        next.is_some() ==> next.unwrap() == Disposition::Released,
{
    match disposition {
        Disposition::Open => Some(Disposition::Released),
        Disposition::Continued | Disposition::Released => None,
    }
}

fn reopened_release_cannot_reclaim_twice(expected: u64) {
    let terminal = record_release(Disposition::Open).unwrap();
    assert(terminal == Disposition::Released);
    let reclaim = reclaim_expected(false, Disposition::Released, expected);
    let repeated = record_release(terminal);
    assert(reclaim.is_none());
    assert(repeated.is_none());
}

}
