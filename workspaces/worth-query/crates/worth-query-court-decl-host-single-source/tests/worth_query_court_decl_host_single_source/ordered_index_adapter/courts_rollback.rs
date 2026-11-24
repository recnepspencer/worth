//! Same-host mid-edit warm AVL budget rollback court (Blocker 4).
use super::queries::{task_title, FindFocusOverlap, FindTitlePrefix, ReadTasks};
use super::schema::{DaymarkSchema, FocusIntervalIndex, FocusWindow, TitlePrefixIndex};
use super::title_prefix::TitlePrefixDomain;
use super::{
    create_task, keys, limits, rename_task, seed_workspace, try_rename_task,
    try_rename_task_with_policy, Program,
};
use std::cmp::Ordering;
use std::num::NonZeroUsize;
use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering as AtomicOrdering};
use std::sync::Arc;
use worth_execution::{
    CancellationToken, ExecutionResourceLease, LeaseRequest, MapKernelStop, WorkCeilingDenial,
};
use worth_foundational::facade::{
    DeterminismContract, ExecutionBudget, ExecutionPosture, ExecutionRequestPolicy,
};
use worth_query_decl::facade::application_program::ApplicationProgramAuthoring;
use worth_query_decl::facade::application_query::ApplicationManagedIndexBinding;
use worth_query_decl::facade::application_schema::ApplicationSchema;
use worth_query_host::facade::application_contribution::{
    WorthQueryAdvancementDenial, WorthQueryManagedComputationResourceDenial,
};
use worth_query_host::facade::application_entry::*;
use worth_query_host::facade::application_installation::*;
use worth_query_host::facade::ordered_index::*;
use worth_query_host::facade::primary_graph::*;
use worth_query_host::facade::runtime::ExecutionAllocationPolicy;
use worth_query_testkit_document_retention::facade::document_retention_model::operator_identity::{
    authenticate_operator, request_scope,
};

#[derive(Clone)]
pub struct TitlePassObserver {
    pub observed_pass2_long_title: Arc<AtomicBool>,
    pub pass2_compare_count: Arc<AtomicUsize>,
    pub pass2_stopped_on_work_ceiling: Arc<AtomicBool>,
}

/// Pass-observing wrapper over `TitlePrefixDomain` with zero mocked errors: every stop
/// originates from Query's real `ExecutionBudget` `work_ceiling`.
pub(super) struct ObservedTitlePrefixDomain {
    observer: TitlePassObserver,
}

type ProviderStop = WorthQueryManagedIndexProviderStop<super::title_prefix::TitlePrefixDenial>;

impl ObservedTitlePrefixDomain {
    pub(super) fn new(observer: TitlePassObserver) -> Self {
        Self { observer }
    }

    fn record_if_pass2_work_ceiling<T>(
        &self,
        result: Result<T, ProviderStop>,
    ) -> Result<T, ProviderStop> {
        let is_ceiling = matches!(
            &result,
            Err(WorthQueryManagedIndexProviderStop::Controlled(
                WorthQueryManagedIndexStop::Request(WorkCeilingDenial::Stopped(
                    MapKernelStop::WorkCeiling,
                )) | WorthQueryManagedIndexStop::ValueDecode(
                    WorthQueryValueDecodeDenial::Checkpoint(MapKernelStop::WorkCeiling),
                ),
            ))
        );
        if is_ceiling
            && self
                .observer
                .observed_pass2_long_title
                .load(AtomicOrdering::SeqCst)
        {
            self.observer
                .pass2_stopped_on_work_ceiling
                .store(true, AtomicOrdering::SeqCst);
        }
        result
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
        self.record_if_pass2_work_ceiling(TitlePrefixDomain.compare_rows(left, right, control))
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
        self.record_if_pass2_work_ceiling(TitlePrefixDomain.source_summary(slot, row, control))
    }

    fn combine_summaries(
        &self,
        left: &Self::Summary,
        right: &Self::Summary,
        sources: &WorthQueryOrderedIndexSources<'_, '_, DaymarkSchema, TitlePrefixIndex>,
        control: &WorthQueryManagedIndexControl<'_>,
    ) -> Result<Self::Summary, WorthQueryManagedIndexProviderStop<Self::Denial>> {
        self.record_if_pass2_work_ceiling(
            TitlePrefixDomain.combine_summaries(left, right, sources, control),
        )
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
    let pass2_stopped_on_work_ceiling = Arc::new(AtomicBool::new(false));
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
            pass2_stopped_on_work_ceiling: Arc::clone(&pass2_stopped_on_work_ceiling),
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
    let read_all = || {
        request
            .query(ReadTasks {
                workspace: "main".into(),
            })
            .execute()
            .unwrap()
            .rows()
            .to_vec()
    };
    let pre_rows = read_all();

    // Pass 1 removes "Alpha" cheaply (~1 byte/node mismatch); Pass 2 inserts 45 KiB shared-prefix
    // title under a tighter per-operation work ceiling so WorkCeiling fires strictly inside Pass 2.
    observed_pass2.store(false, AtomicOrdering::SeqCst);
    pass2_compares.store(0, AtomicOrdering::SeqCst);
    pass2_stopped_on_work_ceiling.store(false, AtomicOrdering::SeqCst);
    let pass2_lease = ExecutionResourceLease::serial(LeaseRequest {
        policy: ExecutionRequestPolicy::new(
            ExecutionPosture::Serial,
            DeterminismContract::CanonicalBitwise,
            ExecutionBudget::new(NonZeroUsize::MIN, None, Some(750_000)),
        ),
        cancellation: CancellationToken::new(),
        deadline: None,
    })
    .unwrap();
    let mut colliding_title = shared;
    colliding_title.push('z');
    let refused = try_rename_task_with_policy(
        &host,
        1,
        &colliding_title,
        99,
        ExecutionAllocationPolicy::Execution(&pass2_lease),
    );
    assert!(
        match &refused {
            Err(WorthQueryApplicationRequestMutationDenial::ExecutionRequest(
                WorthQueryAdvancementDenial::Resource(
                    WorthQueryManagedComputationResourceDenial::WorkExhausted,
                ),
            )) => true,
            Ok(WorthQueryApplicationMutationOutcome::Commit(
                WorthQueryApplicationUncommitted::Denied(denial),
            )) => matches!(
                denial.kind(),
                WorthQueryApplicationCommitDenialKind::ExecutionResource {
                    denial: WorthQueryManagedComputationResourceDenial::WorkExhausted,
                    ..
                }
            ),
            _ => false,
        },
        "warm rename must return typed WorkExhausted refusal, got {refused:?}"
    );
    assert!(
        observed_pass2.load(AtomicOrdering::SeqCst)
            && pass2_compares.load(AtomicOrdering::SeqCst) > 0
            && pass2_stopped_on_work_ceiling.load(AtomicOrdering::SeqCst),
        "WorkCeiling stop must occur strictly inside Pass 2 after Pass 1 removed the old slot"
    );

    // Same host retains exact pre-state rows, BOTH indexes, and unwoken watches.
    assert_eq!(read_all(), pre_rows, "authoritative rows must be unchanged");
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
