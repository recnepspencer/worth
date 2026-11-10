//! Fixed commit and Source roster beside independently registered unrelated predicates.
use super::*;

pub(in super::super::super) fn run() {
    let counts = [1, 16, 256].map(sample);
    assert!(counts
        .iter()
        .all(|(work, updates, _)| *work > 0 && *updates > 0));
    assert!(
        counts.windows(2).all(|pair| pair[0] == pair[1]),
        "unrelated retained index queries must add zero commit work: {counts:?}"
    );
}
fn sample(count: usize) -> (u64, usize, usize) {
    let (world, calls, _, mut key) = setup_growth();
    let scope = authentication::request_scope();
    let principal = principal(&world, &scope);
    let request = world
        .runtime()
        .request(&principal, &scope)
        .on_branch(world.current_world());
    let (relevant_initial, mut relevant) = request
        .query(find("A", "match"))
        .watch_index::<prefix::PrefixIndex>()
        .unwrap();
    assert_eq!(names(relevant_initial.rows()), set(&["x"]));
    let mut unrelated = Vec::new();
    for index in 0..count {
        let probe = format!("unrelated-{index:04}");
        let (initial, watch) = request
            .query(find("U", &probe))
            .watch_index::<prefix::PrefixIndex>()
            .unwrap();
        assert!(initial.rows().is_empty());
        unrelated.push((probe, initial, watch));
    }
    assert_eq!(unrelated.len(), count);
    let builds = calls.builds.load(Ordering::SeqCst);
    let updates = calls.updates.load(Ordering::SeqCst);
    let regions = calls.predicate_regions.load(Ordering::SeqCst);
    let ((), work) = super::super::super::court::prefix_growth::facade_invoice(|| {
        publish(
            &world,
            &mut key,
            authoring::Edit::Name {
                item: "x".into(),
                value: "changed".into(),
            },
        )
    });
    let updates = calls.updates.load(Ordering::SeqCst) - updates;
    let regions = calls.predicate_regions.load(Ordering::SeqCst) - regions;
    assert_eq!(
        calls.builds.load(Ordering::SeqCst),
        builds,
        "a warm commit cannot hide a Source reconstruction"
    );
    eprintln!("retained unrelated index queries={count}: SDK={work}, provider updates={updates}, predicate regions={regions}");
    let WorthQueryManagedIndexWatchOutcome::Delivered(changed) = request
        .query(find("A", "match"))
        .next_watch(&mut relevant)
        .unwrap()
    else {
        panic!("the measured commit must deliver its real membership departure");
    };
    assert!(changed.read().rows().is_empty());
    assert_eq!(names(relevant_initial.rows()), set(&["x"]));
    for (probe, initial, mut watch) in unrelated {
        assert!(initial.rows().is_empty());
        assert!(matches!(
            request
                .query(find("U", &probe))
                .next_watch(&mut watch)
                .unwrap(),
            WorthQueryManagedIndexWatchOutcome::Pending
        ));
        watch.close().unwrap();
    }
    relevant.close().unwrap();
    (work, updates, regions)
}
