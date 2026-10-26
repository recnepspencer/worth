//! What a recovered home must hold after its previous incarnation stopped
//! with at most one commit in flight, and what it must do next.
//!
//! The recovered artifacts are the stopped run's acknowledged artifacts plus
//! at most the one in-flight commit, each byte-identical to the reference's
//! at its position (D11); the journal's `resolve` agrees with that; record
//! states equal the reference after the recovered count; the rest of the
//! script then runs to completion on the reference states; and a fresh
//! recovery reaches the same history.

use std::path::Path;

use worth_execution::ExecutionRequest;
use worth_relational::facade::durability::{
    RelationalCanonicalCommitArtifact, RelationalDurabilityUnresolved,
};
use worth_relational::facade::publication::PatchStreamPosition;
use worth_store_relational::RelationalStoreCommitFate;
use worth_store_relational::RelationalStoreHome;

use super::durable_run::{
    apply, assert_holds, assert_reference_artifacts, open_declaring, served_schema, steps_after,
};
use super::identity_safety;
use super::workload::{self, Script};

/// What the stopped run knew about the commit in flight when it stopped.
#[derive(Debug)]
pub(crate) enum InFlight {
    /// Nothing was in flight, or it was acknowledged and is in the
    /// acknowledged artifacts.
    Nothing,
    /// The medium proved it had no effect; it must not be recovered.
    Refused,
    /// Stopped before Store committed it; it must not be recovered.
    NotDurable,
    /// Stopped after Store made it durable; it must be recovered although the
    /// stopped runtime never installed it.
    Durable,
    /// The fate was unresolved; it is recovered exactly when the journal says
    /// it committed, and then it is the artifact the fate names.
    Unresolved(RelationalDurabilityUnresolved),
}

/// Check the recovered home against the stopped run. Returns how many commits
/// it recovered.
pub(crate) fn assert_recovered(
    home: &RelationalStoreHome,
    script: &Script,
    acknowledged: &[Vec<u8>],
    in_flight: &InFlight,
    case: &str,
) -> usize {
    let recovered = workload::artifacts_of(home.runtime());
    let held = acknowledged.len();
    assert!(
        recovered.len() >= held,
        "{case}: an acknowledged commit was lost ({} < {held})",
        recovered.len()
    );
    for (index, (found, expected)) in recovered.iter().zip(acknowledged).enumerate() {
        assert!(found == expected, "{case}: artifact {} differs", index + 1);
    }
    assert_reference_artifacts(&recovered, script, case);
    let extra = recovered.len() - held;
    assert!(
        extra <= 1,
        "{case}: {extra} commits beyond the acknowledged"
    );
    let next = held as u64 + 1;
    let digest_of = |bytes: &Vec<u8>| {
        RelationalCanonicalCommitArtifact::from_untrusted_bytes(bytes.clone())
            .expect("an artifact frame")
            .payload_digest()
    };
    // Any commit's digest probes presence; the recovered one's proves identity.
    let probe = recovered
        .get(held)
        .map(digest_of)
        .or(match &in_flight {
            InFlight::Unresolved(fate) => Some(fate.digest()),
            _ => None,
        })
        .or_else(|| acknowledged.last().map(digest_of));
    if let Some(probe) = probe {
        let resolution = home
            .resolve(PatchStreamPosition(next), probe)
            .unwrap_or_else(|refusal| panic!("{case}: resolve {next}: {refusal:?}"));
        let expected = match recovered.get(held) {
            Some(_) => RelationalStoreCommitFate::Committed,
            None => RelationalStoreCommitFate::NotCommitted { head: held as u64 },
        };
        assert_eq!(resolution, expected, "{case}: resolve {next}");
    }
    match in_flight {
        InFlight::Nothing | InFlight::Refused | InFlight::NotDurable => {
            assert_eq!(extra, 0, "{case}: {in_flight:?} was recovered")
        }
        InFlight::Durable => assert_eq!(extra, 1, "{case}: a durable commit was lost"),
        InFlight::Unresolved(fate) => {
            assert_eq!(fate.position().0, next, "{case}: the fate's position");
            if let Some(bytes) = recovered.get(held) {
                let artifact =
                    RelationalCanonicalCommitArtifact::from_untrusted_bytes(bytes.clone())
                        .expect("a recovered artifact frame");
                assert_eq!(artifact.commit_id(), fate.commit_id(), "{case}");
                assert_eq!(artifact.payload_digest(), fate.digest(), "{case}");
            }
        }
    }
    let commits = recovered.len();
    assert_eq!(
        workload::states_of(home.runtime(), &script.records),
        script.states[commits],
        "{case}: branch states after {commits} commits"
    );
    commits
}

/// Run the rest of the script on the recovered home, require the reference
/// states and the recovered history as its prefix, then require a fresh
/// recovery to agree and the next creates to issue fresh identities.
pub(crate) fn assert_continues(
    location: &Path,
    home: RelationalStoreHome,
    script: &Script,
    recovered: usize,
    execution: &ExecutionRequest<'_, '_>,
    case: &str,
) {
    let before = workload::artifacts_of(home.runtime());
    apply(&home, execution, steps_after(script, recovered));
    let everything = workload::artifacts_of(home.runtime());
    assert!(
        everything[..recovered] == before[..],
        "{case}: history moved"
    );
    assert_holds(&home, script, &everything);
    home.close();
    let again = open_declaring(location, served_schema(&script.steps), execution)
        .unwrap_or_else(|refusal| panic!("{case}: reopen: {refusal:?}"));
    assert_holds(&again, script, &everything);
    identity_safety::assert_creates_are_fresh(&again, script, execution, case);
    again.close();
}
