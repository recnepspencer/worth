//! Each request is checked against signed postings computed from inputs.
use super::{
    journal_model::Journal,
    observation::{Observer, Posture},
    seeded_world::{World, SEED},
};
fn run(posture: Posture) {
    let _owned=worth_query_host::facade::primary_graph::WorthQueryExecutionObservation::install_on_this_thread();
    let world = World::new(SEED);
    let mut model = Journal::new(&[(11, 1), (12, 2)], 100, 11, world.funding);
    let mut observer = Observer::new(&world);
    let mut bill = (0_u64, 0_u64);
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
        println!("BANK_CELL:{place}:{charged:?}");
        for record in records {
            bill.0 += record.report.charged_work();
            bill.1 += record.report.charged_span();
        }
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
    // Two new commits cost (28+1578)/(28+1552); three noncommits cost 2*28 each.
    assert_eq!(
        bill,
        (3_380, 3_328),
        "absolute Bank history work and span price the five declared requests"
    );
}

#[test]
fn seeded_transfers_match_at_host_policy_and_each_request_width() {
    if let Ok(width) = std::env::var("WORTH_BANK_HISTORY_WIDTH") {
        let width = width.parse::<usize>().unwrap();
        run(std::num::NonZeroUsize::new(width).map_or(Posture::Serial, Posture::Leased));
        return;
    }
    let mut baseline = None;
    let mut red = Vec::new();
    for width in [0, 1, 2, 5] {
        let output=std::process::Command::new(std::env::current_exe().unwrap()).args(["--exact","parallel_history::transfer_history::seeded_transfers_match_at_host_policy_and_each_request_width","--nocapture"]).env("WORTH_BANK_HISTORY_WIDTH",width.to_string()).output().unwrap();
        let stdout = String::from_utf8_lossy(&output.stdout);
        assert!(
            output.status.success(),
            "Bank width {width}: {stdout}{}",
            String::from_utf8_lossy(&output.stderr)
        );
        let lines = stdout
            .lines()
            .filter(|line| line.starts_with("BANK_CELL:"))
            .collect::<Vec<_>>();
        assert_eq!(lines.len(), 5, "five observed transfer steps at each width");
        let records = stdout
            .lines()
            .filter(|s| s.starts_with("BANK_CELL:"))
            .collect::<Vec<_>>()
            .join("\n");
        if let Some(expected) = &baseline {
            if expected != &records {
                red.push(format!("width {width}: {expected} versus {records}"));
            }
        } else {
            baseline = Some(records);
        }
    }
    for finding in &red {
        println!("BANK_NAMED_RED:{finding}");
    }
    assert!(
        red.is_empty(),
        "Bank results, charged work and commit sequence agree across request widths"
    );
}
