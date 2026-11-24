//! #361: a coupled producer reads persistent groups on the envelope's
//! unpublished candidate. Its Initial, run by model `a`'s edit, reads the
//! loop holding a member and resolves the held loop in `decide`; the build
//! receives the loop it resolved, the producer seals, and the envelope
//! commits with the groups exactly as the read answered.
use super::super::super::groups::{declare, group, loop_of, loops};
use super::group_reads::{observed, read_groups, Observed};
use super::*;
use GroupRole::{Host, Member as Plain};

#[test]
fn a_coupled_producer_reads_groups_on_the_candidate_and_builds_with_them() {
    let (token, value) = ("producer-groups", 3201);
    let world = World::install();
    let n = world.names(&format!("{token}-basis"), &["direct", "nested"], 0);
    declare(
        &world,
        &format!("{token}-loop"),
        loops(vec![group(&[(Host, n[0]), (Plain, n[1])])], &[]),
    );
    let whole = loop_of(&world, &format!("{token}-whole"), n[0]).expect("a loop");
    read_groups(value, n[1], whole);
    let outcome = world.shape("a", value, false, token);
    assert!(
        matches!(outcome, outputs::Outcome::Committed { .. }),
        "{outcome:?}"
    );
    assert_eq!(
        observed(value),
        Some(Observed {
            loop_of: Ok(Some(whole)),
            resolved: Ok(Some(whole)),
            built_with: Some(whole),
        }),
        "both reads answered from the commit basis, on the candidate"
    );
    assert!(
        world.live(&primary_key("a")),
        "the producer's outputs committed"
    );
    assert_eq!(
        loop_of(&world, &format!("{token}-after"), n[1]),
        Some(whole),
        "the committed groups agree with the candidate read"
    );
}
