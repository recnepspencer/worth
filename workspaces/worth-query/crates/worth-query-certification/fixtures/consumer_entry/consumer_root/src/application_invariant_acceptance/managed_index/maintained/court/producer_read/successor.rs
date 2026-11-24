//! A source-owned producer derives again inside one edit: a later member that
//! changes what its aligned read saw reopens it as a successor round of the
//! same route. That round probes the applied candidate. A probe of a scope
//! its chain did not write is independent and commits with the new result; a
//! probe that would read a record its own earlier round wrote refuses before
//! any row is returned, and the round stops as `PriorEffectRead`.
use super::later_append::{item, staged, Later};
use super::super::super::peer_scope::PRODUCER_TRIGGER;
use super::*;

/// As `staged`, in a world whose generated producer owns its route.
fn owned(
    probe: ProducerProbe,
    computation: bool,
    earlier: Option<Edit>,
    later: Edit,
    directory: &'static str,
    key: u64,
) -> Later {
    super::super::super::generated::producer::SOURCE_OWNED.set(true);
    let run = staged(probe, computation, earlier, later, directory, key);
    super::super::super::generated::producer::SOURCE_OWNED.set(false);
    run
}

/// A later stage that rewrites the note the producer's decision read: the
/// producer derives again without the stage touching any probed Source.
fn retrigger() -> Edit {
    Edit::Note {
        directory: "A".into(),
        note: "changed".into(),
    }
}

const POSTPONED: &str =
    "CoupledContinuationProvenancePostponed { issue: 119, reason: PriorEffectRead }";

#[test]
fn a_reopened_round_probing_a_scope_its_chain_did_not_write_commits() {
    lifecycle::run(|| {
        PRODUCER_TRIGGER.set(Some("A"));
        // The peer is untouched by the envelope: both rounds take the
        // committed index, and the second round's read keeps its support.
        let aligned = owned(
            ProducerProbe::Peer("peer"),
            false,
            None,
            retrigger(),
            "peer",
            0x3104_0010,
        );
        // An earlier stage changed the peer: both rounds probe the applied
        // candidate, and the second round reads no record of its own chain.
        let applied = owned(
            ProducerProbe::Peer("peer"),
            false,
            Some(item("peer")),
            retrigger(),
            "peer",
            0x3104_0020,
        );
        PRODUCER_TRIGGER.set(None);
        // Two observations each: the producer really derived a second round.
        assert_eq!(
            aligned.observed,
            vec![Ok(vec!["p".to_owned()]), Ok(vec!["p".to_owned()])],
            "{}",
            aligned.detail
        );
        assert!(aligned.committed, "{}", aligned.detail);
        let both = vec!["late".to_owned(), "p".to_owned()];
        assert_eq!(applied.observed, vec![Ok(both.clone()), Ok(both)], "{}", applied.detail);
        assert!(applied.committed, "{}", applied.detail);
        assert_eq!(applied.rows, ["late", "p"]);
    });
}

#[test]
fn a_reopened_round_probing_its_own_chain_output_stops_as_prior_effect_read() {
    lifecycle::run(|| {
        let run = owned(ProducerProbe::Own, false, None, item("catalog"), "catalog", 0x3104_0040);
        // Round one read its own unchanged Directory and published an Item
        // into it. The later stage reopened it; the second round's probe
        // would read that Item as its own chain left it, so it refuses and
        // the edit stops typed, publishing nothing.
        assert_eq!(
            run.observed,
            vec![
                Ok(vec!["c".to_owned()]),
                Err(ProbeRefusal::Refused(
                    WorthQueryManagedIndexRefusal::PriorEffectRead
                )),
            ],
            "{}",
            run.detail
        );
        assert!(!run.committed);
        assert!(run.detail.contains(POSTPONED), "{}", run.detail);
        assert_eq!(run.rows, ["c"]);
    });
}

#[test]
fn an_aligned_peer_read_reopens_and_rederives_when_a_later_stage_edits_the_peer() {
    lifecycle::run(|| {
        let run = owned(ProducerProbe::Peer("peer"), false, None, item("peer"), "peer", 0x3104_0060);
        // Round one took the committed index for the unchanged peer. The
        // later stage's Item marked that read, qualified use reopened the
        // owned producer, and its second round read the peer on the applied
        // candidate, seeing the new Item: the stale decision is not used.
        assert_eq!(
            run.observed,
            vec![
                Ok(vec!["p".to_owned()]),
                Ok(vec!["late".to_owned(), "p".to_owned()]),
            ],
            "{}",
            run.detail
        );
        // The round then stops. Its chain linked the Source's own relation
        // kind and the peer's list changed after the chain began; the
        // own-output view holds records, not relations (#277), so the
        // round's independence of its own links is not proved.
        assert!(!run.committed);
        assert!(run.detail.contains(POSTPONED), "{}", run.detail);
        assert_eq!(run.rows, ["p"]);
    });
}
