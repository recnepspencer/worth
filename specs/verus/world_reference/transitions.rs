//! proof-mirror: crates/worth-runtime-world/src/identity/branch.rs:70-78 sha256:632d8e80e114d70f91f36fef75f4c9c4aa74f8a08dcb18fa74d0df804bea7f5a
//! proof-mirror: crates/worth-runtime-world/src/durability/artifact/creation.rs:85-86 sha256:a77e232bc649201fecf5cd1d8058e6c9a6f9a18e1507c8378351d95819b49bd8
//! proof-mirror: crates/worth-runtime-world/src/branch/reference_cell/successor_validation.rs:51-57 sha256:5913bbfeff344e060e50dc8c20b10d4a435410c9e722515e3af315abf78e245e
//! proof-mirror: crates/worth-runtime-world/src/durability/readmission/open.rs:16-38 sha256:9be38afccd560df148266a3b110f35f245436489c5d0845be15f9df299c1a003
//! proof-mirror: crates/worth-runtime-world/src/durability/readmission/journal.rs:251-255 sha256:5c9f487a94a4ae2535e5430c529f792b0b153e58740edbebb7cc5011ba0dcdff
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
