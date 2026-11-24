//! Same-host mid-edit warm AVL budget rollback court (Blocker 4).
use super::queries::*;
use super::schema::*;
use super::title_prefix::TitlePrefixDomain;
use super::{create_task, keys, limits, rename_task, seed_workspace, try_rename_task, Program};
use std::cmp::Ordering;
use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering as AtomicOrdering};
use std::sync::Arc;
use worth_query_decl::facade::application_query::ApplicationManagedIndexBinding;
use worth_query_decl::facade::{application_program::*, application_schema::*};
use worth_query_host::facade::application_entry::*;
use worth_query_host::facade::application_installation::*;
use worth_query_host::facade::ordered_index::*;
use worth_query_host::facade::primary_graph::*;
use worth_query_testkit_document_retention::facade::document_retention_model::operator_identity::{
    authenticate_operator, request_scope,
};

#[derive(Clone)]
pub struct TitlePassObserver {
    pub observed_pass2_long_title: Arc<AtomicBool>,
    pub pass2_compare_count: Arc<AtomicUsize>,
}

/// Pass-observing wrapper over `TitlePrefixDomain` with zero mocked errors: every stop
/// originates from Query's real `ExecutionBudget` `work_ceiling`.
pub(super) struct ObservedTitlePrefixDomain {
    observer: TitlePassObserver,
}

impl ObservedTitlePrefixDomain {
    pub(super) fn new(observer: TitlePassObserver) -> Self {
        Self { observer }
    }
}

impl WorthQueryOrderedIndexDomain<DaymarkSchema, TitlePrefixIndex> for ObservedTitlePrefixDomain {
    type Summary = WorthQueryOrderedSlotBounds;
    type Denial = super::title_prefix::TitlePrefixDenial;
    const IDENTITY: &'static str = TitlePrefixIndex::PROVIDER;
    const VERSION: u32 = TitlePrefixIndex::PROVIDER_VERSION;

    fn compare_rows(
        &self,
        left: WorthQueryManagedIndexSourceRow<'_, DaymarkSchema, TitlePrefixIndex>,
        right: WorthQueryManagedIndexSourceRow<'_, DaymarkSchema, TitlePrefixIndex>,
        control: &WorthQueryManagedIndexControl<'_>,
    ) -> Result<Ordering, WorthQueryManagedIndexProviderStop<Self::Denial>> {
        if self
            .observer
            .observed_pass2_long_title
            .load(AtomicOrdering::SeqCst)
        {
            self.observer
                .pass2_compare_count
                .fetch_add(1, AtomicOrdering::SeqCst);
        }
        TitlePrefixDomain.compare_rows(left, right, control)
    }

    fn source_summary(
        &self,
        slot: WorthQueryManagedIndexSourceSlot,
        row: WorthQueryManagedIndexSourceRow<'_, DaymarkSchema, TitlePrefixIndex>,
        control: &WorthQueryManagedIndexControl<'_>,
    ) -> Result<Self::Summary, WorthQueryManagedIndexProviderStop<Self::Denial>> {
        let is_heavy = row
            .with_text(task_title(), |text| {
                Ok::<bool, WorthQueryManagedIndexProviderStop<Self::Denial>>(text.len() >= 40_000)
            })
            .map_err(|err| map_text_read_denial(err, control))??;
        if is_heavy {
            self.observer
                .observed_pass2_long_title
                .store(true, AtomicOrdering::SeqCst);
        }
        TitlePrefixDomain.source_summary(slot, row, control)
    }

    fn combine_summaries(
        &self,
        left: &Self::Summary,
        right: &Self::Summary,
        sources: &WorthQueryOrderedIndexSources<'_, '_, DaymarkSchema, TitlePrefixIndex>,
        control: &WorthQueryManagedIndexControl<'_>,
    ) -> Result<Self::Summary, WorthQueryManagedIndexProviderStop<Self::Denial>> {
        TitlePrefixDomain.combine_summaries(left, right, sources, control)
    }

    fn summary_may_match(
        &self,
        summary: &Self::Summary,
        sources: &WorthQueryOrderedIndexSources<'_, '_, DaymarkSchema, TitlePrefixIndex>,
        probe: &String,
        control: &WorthQueryManagedIndexControl<'_>,
    ) -> Result<bool, WorthQueryManagedIndexProviderStop<Self::Denial>> {
        TitlePrefixDomain.summary_may_match(summary, sources, probe, control)
    }

    fn matches_source(
        &self,
        row: WorthQueryManagedIndexSourceRow<'_, DaymarkSchema, TitlePrefixIndex>,
        probe: &String,
        control: &WorthQueryManagedIndexControl<'_>,
    ) -> Result<bool, WorthQueryManagedIndexProviderStop<Self::Denial>> {
        TitlePrefixDomain.matches_source(row, probe, control)
    }
}

impl WorthQueryOrderedIndexPredicateDomain<DaymarkSchema, TitlePrefixIndex>
    for ObservedTitlePrefixDomain
{
    type PredicateSummary = ();

    fn compare_probes(
        &self,
        left: &String,
        right: &String,
        control: &WorthQueryManagedIndexControl<'_>,
    ) -> Result<Ordering, WorthQueryManagedIndexProviderStop<Self::Denial>> {
        TitlePrefixDomain.compare_probes(left, right, control)
    }

    fn predicate_summary(
        &self,
        probe: &String,
        control: &WorthQueryManagedIndexControl<'_>,
    ) -> Result<(), WorthQueryManagedIndexProviderStop<Self::Denial>> {
        TitlePrefixDomain.predicate_summary(probe, control)
    }

    fn predicate_region_may_match(
        &self,
        summary: &(),
        extrema: [&String; 2],
        row: WorthQueryManagedIndexSourceRow<'_, DaymarkSchema, TitlePrefixIndex>,
        control: &WorthQueryManagedIndexControl<'_>,
    ) -> Result<bool, WorthQueryManagedIndexProviderStop<Self::Denial>> {
        TitlePrefixDomain.predicate_region_may_match(summary, extrema, row, control)
    }
}

fn prefix_q(prefix: &str) -> FindTitlePrefix {
    FindTitlePrefix {
        workspace: "main".into(),
        prefix: prefix.to_owned(),
    }
}

fn focus_q(start: i64, end: i64) -> FindFocusOverlap {
    FindFocusOverlap {
        workspace: "main".into(),
        window: FocusWindow::new(start, end),
    }
}

#[test]
fn mid_edit_warm_avl_work_exhaustion_rolls_back_atomically_on_same_host() {
    let observed_pass2 = Arc::new(AtomicBool::new(false));
    let pass2_compares = Arc::new(AtomicUsize::new(0));
    let declaration = DaymarkSchema::declaration().unwrap();
    let program = ApplicationProgramAuthoring::<DaymarkSchema, Program>::begin()
        .with_schema(&declaration)
        .validated_program()
        .unwrap();
    let host = in_memory_program(
        program,
        declaration,
        (Some(TitlePassObserver {
            observed_pass2_long_title: Arc::clone(&observed_pass2),
            pass2_compare_count: Arc::clone(&pass2_compares),
        }),),
        ApplicationBuildIdentity::parse("7b7b7b7b-7b7b-7b7b-7b7b-7b7b7b7b7b7b").unwrap(),
        limits(None, Some(900_000)),
        |_, graph, installed| seed_workspace(graph, installed, &[]),
    )
    .unwrap();

    let scope = request_scope();
    let principal = authenticate_operator(host.installed_schema(), &scope);
    let request = host.runtime().request(&principal, &scope);
    let title_idx = request
        .query(prefix_q("al"))
        .open_index::<TitlePrefixIndex>()
        .unwrap();
    let focus_idx = request
        .query(focus_q(5, 35))
        .open_index::<FocusIntervalIndex>()
        .unwrap();
    request
        .query(prefix_q("al"))
        .rebuild_index(&title_idx)
        .unwrap();
    request
        .query(focus_q(5, 35))
        .rebuild_index(&focus_idx)
        .unwrap();

    // Seed 6 short tasks first (well below 900_000 work per commit), then promote peers
    // 2..=6 to shared 45 KiB prefixes one rename at a time until a 45 KiB rename of task 1
    // ("Alpha") passes Pass 1 cheaply and exhausts WorkCeiling strictly inside Pass 2.
    for (idem, key, title, start, end) in [
        (1_u64, 1_u64, "Alpha", 10_i64, 30_i64),
        (2, 2, "Beta", 40, 60),
        (3, 3, "Delta", 70, 90),
        (4, 4, "Focus", 100, 120),
        (5, 5, "Gamma", 130, 150),
        (6, 6, "Kappa", 160, 180),
    ] {
        create_task(&host, key, title, start, end, idem);
    }
    let shared = "m".repeat(45_000);
    for (idem, key, suffix) in [
        (10_u64, 2_u64, 'a'),
        (11, 3, 'b'),
        (12, 4, 'c'),
        (13, 5, 'd'),
        (14, 6, 'e'),
    ] {
        let mut long_title = shared.clone();
        long_title.push(suffix);
        if !matches!(
            try_rename_task(&host, key, &long_title, idem),
            Ok(WorthQueryApplicationMutationOutcome::Committed { .. })
        ) {
            break;
        }
    }

    let (init_t, mut title_watch) = request
        .query(prefix_q("al"))
        .watch_index::<TitlePrefixIndex>()
        .unwrap();
    let (init_f, mut focus_watch) = request
        .query(focus_q(5, 35))
        .watch_index::<FocusIntervalIndex>()
        .unwrap();
    assert_eq!(keys(init_t.rows()), [1]);
    assert_eq!(keys(init_f.rows()), [1]);
    let pre_rows = request
        .query(ReadTasks {
            workspace: "main".into(),
        })
        .execute()
        .unwrap()
        .rows()
        .to_vec();

    // Pass 1 removes "Alpha" cheaply; Pass 2 inserts 45 KiB shared-prefix title and hits WorkCeiling.
    observed_pass2.store(false, AtomicOrdering::SeqCst);
    pass2_compares.store(0, AtomicOrdering::SeqCst);
    let mut colliding_title = shared;
    colliding_title.push('z');
    let refused = try_rename_task(&host, 1, &colliding_title, 99);
    let detail = format!("{refused:?}");
    let is_work_refusal = !matches!(
        refused,
        Ok(WorthQueryApplicationMutationOutcome::Committed { .. })
    ) && (detail.contains("WorkCeiling") || detail.contains("WorkExhausted"));
    assert!(
        is_work_refusal,
        "warm rename must stop with WorkCeiling/WorkExhausted, got {detail}"
    );
    assert!(
        observed_pass2.load(AtomicOrdering::SeqCst)
            && pass2_compares.load(AtomicOrdering::SeqCst) > 0,
        "stop must occur strictly inside Pass 2 after Pass 1 removed the old slot"
    );

    // Same host retains exact pre-state rows, BOTH indexes, and unwoken watches.
    let post_rows = request
        .query(ReadTasks {
            workspace: "main".into(),
        })
        .execute()
        .unwrap()
        .rows()
        .to_vec();
    assert_eq!(post_rows, pre_rows, "authoritative rows must be unchanged");
    assert_eq!(
        keys(request.query(prefix_q("al")).execute().unwrap().rows()),
        [1]
    );
    assert_eq!(
        keys(request.query(focus_q(5, 35)).execute().unwrap().rows()),
        [1]
    );
    assert!(matches!(
        request
            .query(prefix_q("al"))
            .next_watch(&mut title_watch)
            .unwrap(),
        WorthQueryManagedIndexWatchOutcome::Pending
    ));
    assert!(matches!(
        request
            .query(focus_q(5, 35))
            .next_watch(&mut focus_watch)
            .unwrap(),
        WorthQueryManagedIndexWatchOutcome::Pending
    ));

    // Subsequent valid edit on the SAME host commits cleanly and delivers to the watch.
    rename_task(&host, 1, "Alder", 100);
    let WorthQueryManagedIndexWatchOutcome::Delivered(tw) = request
        .query(prefix_q("al"))
        .next_watch(&mut title_watch)
        .unwrap()
    else {
        panic!("title watch must deliver after subsequent valid rename");
    };
    assert_eq!(keys(tw.read().rows()), [1]);

    title_watch.close().unwrap();
    focus_watch.close().unwrap();
    title_idx.close();
    focus_idx.close();
}
