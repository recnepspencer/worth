//! Each request is checked against signed postings computed from inputs.
pub use super::{
    journal_model::Journal,
    observation::{Observer, Posture},
    seeded_world::{World, SEED},
};
pub fn run(posture: Posture) {
    let _owned=worth_query_host::facade::primary_graph::WorthQueryExecutionObservation::install_on_this_thread();
    let world = World::new(SEED);
    let mut model = Journal::new(&[(11, 1), (12, 2)], 100, 11, world.funding);
    let mut observer = Observer::new(&world);
    for (place, input) in world.requests(SEED).iter().enumerate() {
        let expected = model.apply(input);
        let actual = observer.transfer(input, posture);
        assert_eq!(
            actual.as_ref().ok(), Some(&expected),
            "request {place}: declared commit, authorization, funds and retry outcome; native {actual:?}"
        );
        let records = &observer.last;
        let expected_events = if [0, 4].contains(&place) { 2 } else { 0 };
        assert_eq!(
            records
                .iter()
                .map(|record| record.applied.len())
                .sum::<usize>(),
            expected_events,
            "the declared transfer history has two first commits and no retry/denial events"
        );
        for record in records {
            for pair in record.applied.chunks(2) {
                assert_eq!(pair.len(), 2, "Bank commit/publication pair");
                assert_eq!(pair[0].kind,worth_query_host::facade::primary_graph::WorthQueryAdvancementEventKindForTest::Commit);
                assert_eq!(pair[1].kind,worth_query_host::facade::primary_graph::WorthQueryAdvancementEventKindForTest::Publication);
                assert_eq!(
                    pair[0].identity, pair[1].identity,
                    "each publication carries its own commit identity"
                );
            }
        }
        let charged = records
            .iter()
            .map(|r| {
                (
                    r.report.charged_work(),
                    r.report.charged_span(),
                    r.applied
                        .iter()
                        .map(|e| {
                            use worth_query_host::facade::primary_graph::WorthQueryAdvancementIdentityForTest as Identity;
                            let Identity::Mutation { key, intent, .. } = &e.identity else { panic!("transfers emit mutation identities") };
                            // Package seals are installation-local; these canonical
                            // runtime key/intent identities are compared across widths.
                            (e.kind, *key, *intent)
                        })
                        .collect::<Vec<_>>(),
                )
            })
            .collect::<Vec<_>>();
        println!("\nBANK_CELL:{place}:{charged:?}");
        let (postings, report) = observer.journal();
        assert_eq!(
            postings,
            model.canonical_postings(),
            "request {place}: exact committed postings"
        );
        assert_eq!(
            postings.iter().map(|posting| posting.amount).sum::<i64>(),
            0,
            "journal including cash conserves signed units"
        );
        assert_eq!(
            observer.balances(),
            model.balances,
            "request {place}: public account summary balances"
        );
        assert!(report.observed_read_work > 0);
    }
}
