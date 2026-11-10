//! Facade invoice equality and distinct physical page evidence on real ordered Sources.
use super::*;
use worth_query_host::facade::certification::*;

pub(in super::super::super) fn run() {
    let (world, _, _, mut key) = setup_growth();
    for i in 0..64 {
        publish(
            &world,
            &mut key,
            authoring::Edit::Item {
                directory: "A".into(),
                identity: format!("seed-{i:04}"),
                name: format!("match-{i:04}"),
                root: true,
                incarnation: 1,
            },
        );
    }
    let before = world
        .runtime()
        .observe_certification_managed_index_pages()
        .unwrap();
    let mut previous = "D".to_owned();
    for i in 0..4 {
        let event = format!("evidence-{i}");
        let (((), facade), internal) = super::super::super::court::prefix_growth::invoice(|| {
            super::super::super::court::prefix_growth::facade_invoice(|| {
                edit_history(&world, &mut key, append(&event, &previous, None, None))
            })
        });
        assert!(internal > 0);
        assert_eq!(
            facade, internal,
            "Host evidence must be the SDK's original invoice"
        );
        eprintln!("prefix facade/internal invoice scope={event}: {facade}/{internal}");
        previous = event;
    }
    let after = world
        .runtime()
        .observe_certification_managed_index_pages()
        .unwrap();
    assert_eq!(before.len(), 1);
    assert_eq!(after.len(), 1);
    let (before, after) = (&before[0], &after[0]);
    assert_eq!(before.program, after.program);
    assert!(after.retained_roots > before.retained_roots);
    assert!(
        after.distinct_pages > 1,
        "the fixture must span multiple real pages"
    );
    assert!(
        after.distinct_pages < after.page_references,
        "unchanged ordered scopes must share genuine physical pages: {after:?}"
    );
    let forked = world
        .runtime()
        .with_host_advancement(|phase| {
            world
                .runtime()
                .branches()
                .fork(world.current_world())
                .components(|c| c.fork_relational().reuse_exact_signal_basis())
                .create(&phase)
                .unwrap()
        })
        .unwrap();
    let shared = world
        .runtime()
        .observe_certification_managed_index_pages()
        .unwrap();
    assert_eq!(shared.len(), 1);
    assert_eq!(
        shared[0].distinct_pages, after.distinct_pages,
        "forking exact immutable material adds no physical node page"
    );
    assert_eq!(shared[0].retained_roots, after.retained_roots * 2);
    assert_eq!(shared[0].page_references, after.page_references * 2);
    let _retained_fork = forked;
    eprintln!(
        "prefix retained roots={} distinct pages={} page references={}",
        after.retained_roots, after.distinct_pages, after.page_references
    );
}
