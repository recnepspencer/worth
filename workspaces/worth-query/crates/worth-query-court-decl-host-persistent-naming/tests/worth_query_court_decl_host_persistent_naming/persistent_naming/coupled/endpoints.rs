//! A later member may link a named relation to a record an earlier member
//! of the same envelope created. The relation's planned birth must name the
//! created endpoint exactly as the composed commands do, under every symbol
//! policy, so the envelope commits the name a direct operation mints.
use super::*;

fn first() -> Plan {
    Plan {
        creates: keys(&["m1"]),
        steps: vec![Step::Birth("m1".into(), birth("cross-m1", "authored"))],
        ..Plan::default()
    }
}

/// Links a named seam from `m1`, which the first stage created.
fn second() -> Plan {
    Plan {
        plain_reads: keys(&["m1"]),
        creates: keys(&["m2"]),
        seams: vec![("cross".into(), "m1".into(), "m2".into())],
        steps: vec![
            Step::Birth("m2".into(), birth("cross-m2", "authored")),
            Step::SeamBirth("cross".into(), birth("cross-seam", "seam_piece")),
        ],
        ..Plan::default()
    }
}

fn minted(world: &World, token: &str) -> Vec<String> {
    probe::seams::name(token, "cross", "m1", "m2");
    let (seams, _) = world.probe_seams(token, &["m1", "m2"], 0);
    let mut names: Vec<String> = world
        .names(token, &["m1", "m2"], 1)
        .iter()
        .map(|name| format!("{name:?}"))
        .collect();
    names.push(format!("{:?}", outputs::seam_named(&seams, "cross")));
    names
}

#[test]
fn a_later_stage_names_a_seam_ending_on_an_earlier_stages_member() {
    let direct = World::install();
    direct.committed("cross-direct-1", first());
    direct.committed("cross-direct-2", second());
    let expected = minted(&direct, "cross-direct-names");

    let coupled = World::install();
    let outcome = coupled.envelope(
        "cross-envelope",
        vec![
            stage("cross-envelope-1", first()),
            stage("cross-envelope-2", second()),
        ],
    );
    assert!(
        matches!(outcome, Envelope::Committed(_)),
        "the seam's birth names the earlier stage's member: {outcome:?}"
    );
    assert_eq!(minted(&coupled, "cross-coupled-names"), expected);
}
