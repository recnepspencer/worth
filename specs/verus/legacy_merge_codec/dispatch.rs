//! proof-mirror: crates/worth-relational-contracts/src/durability/canonical_commit_artifact/frame.rs:140-163 sha256:61b0088053bb14ff48287ae15529d1abc4829959965be66a47be9d03017125cc
//! proof-mirror: crates/worth-relational-durability/src/durability/log/native_checkpoint_codec.rs:25-52 sha256:594e87d1187a26633c7aee52a2aa58e83cc6379be2f8dfa91314b4c243df496b
//! proof-mirror: crates/worth-relational-durability/src/durability/canonical_commit_artifact/decoded_quote/envelope.rs:48-143 sha256:7b661f465105e8eaa7f29332c78ec1cf59da541652216e26a9ec3a76d9c95d80
//! The frame decoder supplies supported-version and legacy-slot facts from
//! the authenticated bytes. This model proves the ensuing format decision.

use vstd::prelude::*;

verus! {

#[derive(PartialEq, Eq)]
enum Disposition {
    InspectOrdinary,
    UnsupportedVersion,
    RetiredMerge,
    UnexpectedLegacySlot,
}

fn dispatch(
    supported: bool,
    legacy: bool,
    legacy_slot: bool,
    populated: bool,
) -> (outcome: Disposition)
    ensures
        outcome == Disposition::UnsupportedVersion <==> !supported,
        outcome == Disposition::RetiredMerge <==> supported && legacy_slot && populated,
        outcome == Disposition::UnexpectedLegacySlot <==>
            supported && legacy_slot && !populated && !legacy,
        outcome == Disposition::InspectOrdinary <==>
            supported && (!legacy_slot || legacy && !populated),
{
    if !supported {
        Disposition::UnsupportedVersion
    } else if legacy_slot && populated {
        Disposition::RetiredMerge
    } else if legacy_slot && !legacy {
        Disposition::UnexpectedLegacySlot
    } else {
        Disposition::InspectOrdinary
    }
}

}
