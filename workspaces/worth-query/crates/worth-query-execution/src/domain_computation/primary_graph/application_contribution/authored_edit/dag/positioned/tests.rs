//! Pure barrier closure over declared edges; no schema or producer is run.
use super::*;

/// Two stages (0, 1), then producers 2.. named by `families`; every family is
/// declared. Resolves stage 1's barrier with `pending` left by stage 0.
fn barrier(
    families: &[&'static str],
    pending: &[&str],
    overlaps: &[(usize, usize)],
    declared: &[(usize, usize)],
) -> Vec<String> {
    let family = |i: usize| families[i - 2];
    let overlaps = |reader: usize, writer: usize| overlaps.contains(&(reader, writer));
    let declared = |reader: usize, writer: usize| declared.contains(&(reader, writer));
    let planned = plan::<()>(
        Closure {
            stages: 2,
            nodes: 2 + families.len(),
            family: &family,
            overlaps: &overlaps,
            declared: &declared,
        },
        || Ok(()),
    )
    .unwrap();
    planned[1].resolve(pending)
}

#[test]
fn a_later_producers_inputs_are_not_the_stage_barrier() {
    // Stage 1 reads what the later producer `f` writes; `f` reads what the
    // pending `a` writes. `f` has not run, so `a` is not stage 1's prerequisite.
    let overlap = barrier(&["a", "f"], &["a"], &[(1, 3), (3, 2)], &[]);
    assert_eq!(overlap, Vec::<String>::new());
    let connected = barrier(&["a", "f"], &["a"], &[(1, 3)], &[(3, 2)]);
    assert_eq!(connected, Vec::<String>::new());
}

#[test]
fn a_pending_producer_read_directly_or_through_another_stays_in_the_barrier() {
    let direct = barrier(&["a", "f"], &["a"], &[(1, 2), (1, 3)], &[]);
    assert_eq!(direct, ["a"]);
    // Stage 1 reads `b`; `b` reads `a` by overlap, and `c` feeds `a` by connection.
    let chained = barrier(
        &["a", "b", "c"],
        &["a", "b", "c"],
        &[(1, 3), (3, 2)],
        &[(2, 4)],
    );
    assert_eq!(chained, ["b", "a", "c"]);
}

#[test]
fn a_declared_family_with_no_pending_root_is_not_in_the_barrier() {
    // Stage 1 reads `a` directly, and `b` through `a`. Both are declared.
    let reads = [(1, 2), (2, 3)];
    assert_eq!(barrier(&["a", "b"], &["a", "b"], &reads, &[]), ["a", "b"]);
    // Nothing pending: declarations alone settle nothing.
    assert_eq!(barrier(&["a", "b"], &[], &reads, &[]), Vec::<String>::new());
    // `a` pending, `b` only declared: `a` waits, `b` has nothing to settle.
    assert_eq!(barrier(&["a", "b"], &["a"], &reads, &[]), ["a"]);
    // `b` pending behind an unpending `a`: `a` does not run, so `b` is not
    // stage 1's prerequisite either.
    assert_eq!(
        barrier(&["a", "b"], &["b"], &reads, &[]),
        Vec::<String>::new()
    );
}
