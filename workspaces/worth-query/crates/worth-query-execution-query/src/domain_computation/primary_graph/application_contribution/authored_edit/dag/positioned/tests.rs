//! Pure barrier closure over declared edges; no schema or producer is run.
use super::*;

/// Two stages (0, 1), then producers 2.. named by `families`. Stage 0 makes
/// every producer family listed in `first` available to stage 1.
fn barriers(
    families: &[&'static str],
    first: &[&'static str],
    overlaps: &[(usize, usize)],
    declared: &[(usize, usize)],
) -> Vec<Vec<String>> {
    let family = |i: usize| families[i - 2];
    let available = |position: usize| {
        if position == 0 {
            vec![]
        } else {
            first.to_vec()
        }
    };
    let overlaps = |reader: usize, writer: usize| overlaps.contains(&(reader, writer));
    let declared = |reader: usize, writer: usize| declared.contains(&(reader, writer));
    close::<()>(
        Closure {
            stages: 2,
            nodes: 2 + families.len(),
            family: &family,
            available: &available,
            overlaps: &overlaps,
            declared: &declared,
        },
        || Ok(()),
    )
    .unwrap()
}

#[test]
fn a_later_producers_inputs_are_not_the_stage_barrier() {
    // Stage 1 reads what the later producer `f` writes; `f` reads what the
    // available `a` writes. `f` has not run, so `a` is not stage 1's prerequisite.
    let overlap = barriers(&["a", "f"], &["a"], &[(1, 3), (3, 2)], &[]);
    assert_eq!(overlap[1], Vec::<String>::new());
    let connected = barriers(&["a", "f"], &["a"], &[(1, 3)], &[(3, 2)]);
    assert_eq!(connected[1], Vec::<String>::new());
}

#[test]
fn an_available_producer_read_directly_or_through_another_stays_in_the_barrier() {
    let direct = barriers(&["a", "f"], &["a"], &[(1, 2), (1, 3)], &[]);
    assert_eq!(direct[1], ["a"]);
    // Stage 1 reads `b`; `b` reads `a` by overlap, and `c` feeds `a` by connection.
    let chained = barriers(
        &["a", "b", "c"],
        &["a", "b", "c"],
        &[(1, 3), (3, 2)],
        &[(2, 4)],
    );
    assert_eq!(chained[1], ["b", "a", "c"]);
    // Nothing is available before the first stage.
    assert_eq!(direct[0], Vec::<String>::new());
}
