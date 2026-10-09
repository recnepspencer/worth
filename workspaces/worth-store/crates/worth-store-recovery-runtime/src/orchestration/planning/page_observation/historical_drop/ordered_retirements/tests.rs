use super::super::retirement_fixture::{image, image_beside_stray, record, retirement, step};
use super::*;

const WRITER: u8 = 0x31;
const RETIRING: u8 = 0x52;
const LATER: u8 = 0x53;

fn index(
    members: &[(u8, &PersistedPhysicalRecoveryProjection)],
    ordinary: &[StepIdentity],
    routes: &[u64],
) -> OrderedRetirements {
    OrderedRetirements::index(
        members
            .iter()
            .map(|(seed, projection)| (step(*seed), *projection)),
        ordinary.iter().copied(),
        routes.iter().copied().map(record),
    )
}

#[test]
fn an_image_is_emptied_only_when_ordered_edges_retired_every_record_it_holds() {
    let (written, target) = image(2, 3);
    let (first, later) = (retirement(&[1, 2]), retirement(&[3]));
    let members = [(WRITER, &written), (RETIRING, &first), (LATER, &later)];
    let edges = [step(WRITER), step(RETIRING), step(LATER)];
    assert!(index(&members, &edges, &[9]).emptied(&target));
    // A record the selected root still routes was never retired, whichever
    // record of the page it is.
    for routed in 1..=3 {
        assert!(!index(&members, &edges, &[routed]).emptied(&target));
    }
    // Retirements that cover only some records leave the page live.
    assert!(!index(&members[..2], &edges, &[]).emptied(&target));
    let only_first = retirement(&[1]);
    assert!(!index(&[(WRITER, &written), (RETIRING, &only_first)], &edges, &[]).emptied(&target));
    let only_last = retirement(&[3]);
    assert!(!index(&[(WRITER, &written), (LATER, &only_last)], &edges, &[]).emptied(&target));
    // Unrouted alone is never enough.
    assert!(!index(&members[..1], &edges, &[]).emptied(&target));
}

#[test]
fn a_retiring_member_no_ordinary_edge_binds_retires_nothing() {
    let (written, target) = image(2, 3);
    let (first, later) = (retirement(&[1, 2]), retirement(&[3]));
    let members = [(WRITER, &written), (RETIRING, &first), (LATER, &later)];
    // The history holds no ordinary edge of the last retirement.
    assert!(!index(&members, &[step(WRITER), step(RETIRING)], &[]).emptied(&target));
    assert!(!index(&members, &[step(WRITER), step(LATER)], &[]).emptied(&target));
    assert!(!index(&members, &[], &[]).emptied(&target));
    // An edge that differs in any part of the member identity binds nothing.
    for forged in [
        StepIdentity {
            redo_sha256: [0xee; 32],
            ..step(LATER)
        },
        StepIdentity {
            group: step(0x99).group,
            ..step(LATER)
        },
        StepIdentity {
            fate: RecoveryOperationFate::AcknowledgedDurable,
            ..step(LATER)
        },
        StepIdentity {
            operation: [0x99; 32],
            ..step(LATER)
        },
    ] {
        let edges = [step(WRITER), step(RETIRING), forged];
        assert!(!index(&members, &edges, &[]).emptied(&target));
    }
}

#[test]
fn a_record_placed_on_another_generation_of_the_page_is_not_on_the_image() {
    // The member framed generation 2 holding records 1 and 2, and placed
    // record 8 on generation 3 of the same page.
    let (written, target) = image_beside_stray(2, 2, (3, 8));
    let retiring = retirement(&[1, 2]);
    let members = [(WRITER, &written), (RETIRING, &retiring)];
    let edges = [step(WRITER), step(RETIRING)];
    assert!(index(&members, &edges, &[]).emptied(&target));
    // Record 8 is not this image's to keep live, routed or not.
    assert!(index(&members, &edges, &[8]).emptied(&target));
}

#[test]
fn only_the_single_writer_of_an_exact_image_says_what_it_holds() {
    let (written, target) = image(2, 2);
    let retiring = retirement(&[1, 2, 3]);
    let edges = [step(WRITER), step(RETIRING)];
    assert!(index(&[(WRITER, &written), (RETIRING, &retiring)], &edges, &[]).emptied(&target));
    // The writer need not be an ordered edge here; physics requires that.
    assert!(index(
        &[(WRITER, &written), (RETIRING, &retiring)],
        &[step(RETIRING)],
        &[]
    )
    .emptied(&target));
    // Two members that wrote the same image are ambiguous.
    let (again, _) = image(2, 2);
    assert!(!index(
        &[(WRITER, &written), (LATER, &again), (RETIRING, &retiring)],
        &edges,
        &[]
    )
    .emptied(&target));
    // An image of another page generation is not the one the member wrote.
    let (_, newer) = image(3, 2);
    assert!(!index(&[(WRITER, &written), (RETIRING, &retiring)], &edges, &[]).emptied(&newer));
    // An image its writer placed nothing on proves no retirement.
    let (bare, bare_target) = image(3, 0);
    assert!(!index(
        &[(WRITER, &written), (LATER, &bare), (RETIRING, &retiring)],
        &edges,
        &[]
    )
    .emptied(&bare_target));
    // The newer image holds a record the retirement does not name.
    let (grown, grown_target) = image(3, 4);
    assert!(!index(
        &[(WRITER, &written), (LATER, &grown), (RETIRING, &retiring)],
        &edges,
        &[]
    )
    .emptied(&grown_target));
}
