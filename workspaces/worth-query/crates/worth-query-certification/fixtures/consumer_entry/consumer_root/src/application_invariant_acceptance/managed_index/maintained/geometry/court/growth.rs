//! Real populations and whole original SDK invoices, with a separate coordinate oracle.
use super::*;
use worth_execution::{
    CancellationSource, ExecutionWorkCeiling, SerialMemoryBudget, SerialRequest,
};
mod selection;

const REMOTE_BATCH_EDGES: u32 = 1_250;

fn invoice<T>(operation: impl FnOnce() -> T) -> (T, u64) {
    let original = SerialRequest::from_memory(
        SerialMemoryBudget::new(None),
        CancellationSource::new().token(),
        None,
    );
    let (result, report) = ExecutionWorkCeiling::new(None)
        .run_with_checkpoint(None, Some(&original), |_| operation())
        .unwrap();
    (result, report.charged_work())
}

#[test]
fn source_structure_warm_work_keeps_local_growth_on_real_small_chains() {
    let small = journey(16, 0);
    let large = journey(32, 0);
    eprintln!("Source structure local invoices16={small:?}32={large:?}");
    for (first, doubled) in [(small.0, large.0), (small.1, large.1)] {
        assert!(
            doubled <= first * 5 / 4,
            "local structural admission must keep the original invoice's bounded growth"
        );
    }
}

#[test]
fn geometry_chain_growth_uses_real_source_and_original_sdk_invoice() {
    let small = journey(1_000, 0);
    let large = journey(2_000, 0);
    eprintln!("geometry costs1000={small:?} costs2000={large:?}");
    assert!(
        large.0 <= small.0 * 5 / 4,
        "refusal must follow height, not Source size"
    );
    assert!(
        large.1 <= small.1 * 5 / 4,
        "commit must follow height, not Source size"
    );
    let remote = journey(2_000, 50_000);
    eprintln!("geometry costs2000+50000={remote:?}");
    for (base, doubled, beside) in [(small.0, large.0, remote.0), (small.1, large.1, remote.1)] {
        assert!(
            beside <= doubled * 3 / 2,
            "unrelated geometry cannot dominate the whole invoice"
        );
        // Fifty thousand leaves add fewer than six binary height steps. Allow
        // eight times one step and a 20k granule for fixed-width paths whose
        // ordered-node split/grant charge need not be monotone at two samples.
        let one_step = doubled.saturating_sub(base).max(20_000);
        assert!(
            beside.saturating_sub(doubled) <= one_step * 8,
            "the difference of original invoices exposes a full-population sweep"
        );
    }
}

#[test]
fn fixed_geometry_edit_keeps_source_work_local_beside_independent_remote_populations() {
    for unrelated in [0, 256, 1_024] {
        // The same two edited occurrences, independent of the remote axis.
        // journey checks exact +8/+8 Source work and bounded query paths; the
        // whole original invoices remain visible for cost diagnosis.
        let costs = journey(64, unrelated);
        eprintln!("fixed64 remote={unrelated} refusal/commit original SDK invoices={costs:?}");
    }
}

fn journey(vertices: u32, unrelated: u32) -> (u64, u64) {
    let calls = Arc::new(provider::Calls::default());
    let configuration = Configuration::new(Arc::clone(&calls));
    let rules = Arc::clone(&configuration.rules);
    let prefix_calls = Arc::new(prefix::Calls::default());
    let prefix_rules = Arc::new(prefix_rule::Calls::new(Arc::clone(&prefix_calls)));
    let world = host::install_geometry_growth(prefix_calls, prefix_rules, configuration);
    let scope = authentication::request_scope();
    let principal = authenticate(&world, &scope);
    setup(&world, &principal, &scope);
    commit(
        &world,
        &principal,
        &scope,
        Edit::Chain {
            directory: "A".into(),
            shared_directory: None,
            prefix: "chain".into(),
            vertices,
            origin: 1_000,
        },
        0x919900,
    );
    let middle = vertices / 2;
    for (offset, edge) in [middle - 1, middle].into_iter().enumerate() {
        commit(
            &world,
            &principal,
            &scope,
            Edit::Share {
                directory: "B".into(),
                edge: format!("chain-e-{edge}"),
            },
            0x919901 + offset as u64,
        );
    }
    for (batch, start) in (0..unrelated)
        .step_by(REMOTE_BATCH_EDGES as usize)
        .enumerate()
    {
        let batch_scope = authentication::request_scope();
        let batch_principal = authenticate(&world, &batch_scope);
        let count = (unrelated - start).min(REMOTE_BATCH_EDGES);
        let ((), setup_work) = invoice(|| {
            commit(
                &world,
                &batch_principal,
                &batch_scope,
                Edit::Chain {
                    directory: "A".into(),
                    shared_directory: Some("B".into()),
                    prefix: format!("remote-{batch}"),
                    vertices: count + 1,
                    origin: 1_000_000 + i64::from(start) * 4,
                },
                0x919a00 + batch as u64,
            )
        });
        eprintln!("authored remote batch{batch} edges={count}; setup invoice={setup_work}");
    }
    // Each remote batch has its own ordinary scope and authentication. Census
    // and both measured edits share this one fresh context after population.
    let scope = authentication::request_scope();
    let principal = authenticate(&world, &scope);
    let request = world
        .runtime()
        .request(&principal, &scope)
        .on_branch(world.current_world());
    // Census is outside the measured operation and interprets authored geometry.
    for directory in ["A", "B"] {
        let source = request
            .query(query::ReadSource {
                directory: directory.into(),
            })
            .execute()
            .unwrap();
        assert_eq!(
            values(source.rows()),
            universe(directory, vertices, unrelated, None)
        );
    }
    let index = request
        .query(find("A", 50))
        .open_index::<provider::BoxIndex>()
        .unwrap();
    let next_x = 1_000 + i64::from(middle) * 2 + 1;
    let retained = request.query(find("A", next_x)).execute().unwrap();
    let retained_values = intersecting(universe("A", vertices, unrelated, None), next_x);
    assert_eq!(values(retained.rows()), retained_values);
    let original_universe = universe("A", vertices, unrelated, None);
    for x in [50, 1_000, 1_002, next_x, 999_999, 2_000_000] {
        selection::verify(
            &calls,
            &original_universe,
            x,
            "before",
            || request.query(find("A", x)).execute().unwrap(),
            |actual| values(actual.rows()),
        );
    }
    let head = request
        .retain_current_read()
        .unwrap()
        .selected_commit()
        .clone();
    let builds = calls.builds.load(Ordering::SeqCst);
    let updates = calls.updates.load(Ordering::SeqCst);
    let source_bounds = calls.source_bounds.load(Ordering::SeqCst);
    let summary_sources = calls.summary_sources.load(Ordering::SeqCst);
    rules.records.lock().unwrap().clear();
    assert!(!rules.capture_complete_source.load(Ordering::SeqCst));
    let (refusal, refusal_work) = invoice(|| {
        request
            .advanced()
            .mutate(Edit::Move {
                directory: "A".into(),
                vertex: format!("chain-v-{middle}"),
                x: 50,
                y: 0,
            })
            .idempotency(&0x919904u64)
            .execute_in_program(&world, ExecutionAllocationPolicy::SystemAllocation)
            .unwrap()
    });
    require_violation(refusal);
    assert_eq!(
        request.retain_current_read().unwrap().selected_commit(),
        &head
    );
    assert!(calls
        .last_decision
        .lock()
        .unwrap()
        .as_ref()
        .unwrap()
        .is_empty());
    {
        let records = rules.records.lock().unwrap();
        let b = records
            .iter()
            .find(|record| record.after.len() == 4)
            .expect("unopened B actually refuses its four proposed contacts");
        assert_eq!(
            values(b.before.as_ref().unwrap()),
            contacts("B", vertices, unrelated, None)
        );
        assert_eq!(
            values(&b.after),
            contacts("B", vertices, unrelated, Some(50))
        );
        assert!(records
            .iter()
            .all(|record| record.updates_before_callback == updates + 2));
    }
    rules.records.lock().unwrap().clear();
    let ((), commit_work) = invoice(|| {
        commit(
            &world,
            &principal,
            &scope,
            Edit::Move {
                directory: "A".into(),
                vertex: format!("chain-v-{middle}"),
                x: next_x,
                y: 0,
            },
            0x919905,
        )
    });
    assert_ne!(
        request.retain_current_read().unwrap().selected_commit(),
        &head
    );
    assert_eq!(values(retained.rows()), retained_values);
    assert_eq!(
        calls.builds.load(Ordering::SeqCst),
        builds,
        "warm edits never rebuild Source"
    );
    assert_eq!(calls.updates.load(Ordering::SeqCst), updates + 4);
    assert_eq!(
        calls.source_bounds.load(Ordering::SeqCst),
        source_bounds + 8,
        "two changed occurrences per scope and edit, independent of the universe"
    );
    assert_eq!(
        calls.summary_sources.load(Ordering::SeqCst),
        summary_sources + 8,
        "summary qualification revisits only the actual changed Source rows"
    );
    let records = rules.records.lock().unwrap();
    assert_eq!(
        records.len(),
        2,
        "both actual scopes evaluate the successful candidate"
    );
    assert_ne!(
        records[0].scope, records[1].scope,
        "two distinct full-generation scopes"
    );
    drop(records);
    for directory in ["A", "B"] {
        assert_eq!(
            values(request.query(find(directory, 50)).execute().unwrap().rows()),
            contacts(directory, vertices, unrelated, Some(next_x))
        );
        let source = request
            .query(query::ReadSource {
                directory: directory.into(),
            })
            .execute()
            .unwrap();
        assert_eq!(
            values(source.rows()),
            universe(directory, vertices, unrelated, Some(next_x))
        );
    }
    assert_eq!(
        values(request.query(find("A", next_x)).execute().unwrap().rows()),
        intersecting(universe("A", vertices, unrelated, Some(next_x)), next_x)
    );
    let changed_universe = universe("A", vertices, unrelated, Some(next_x));
    for x in [50, 1_000, 1_002, next_x, 999_999, 2_000_000] {
        selection::verify(
            &calls,
            &changed_universe,
            x,
            "after",
            || request.query(find("A", x)).execute().unwrap(),
            |actual| values(actual.rows()),
        );
    }
    assert_eq!(calls.builds.load(Ordering::SeqCst), builds);
    index.close();
    (refusal_work, commit_work)
}

fn authenticate(world: &host::Host, scope: &WorthQueryRequestScope) -> Principal {
    authentication::block_on(
        authentication::admit(world.installed_schema()).authenticate(
            authentication::LocalCredential::issued_for_model_owner(),
            scope,
        ),
    )
    .unwrap()
}

type EdgeValue = (String, Coordinates, Coordinates);
fn universe(directory: &str, vertices: u32, unrelated: u32, moved: Option<i64>) -> Vec<EdgeValue> {
    let point = |x| Coordinates::new(x, 0, x, 0);
    let middle = vertices / 2;
    let mut rows = expected(directory, 10, all(directory).probe);
    for index in 0..vertices - 1 {
        if directory == "B" && index != middle - 1 && index != middle {
            continue;
        }
        let endpoint = |vertex| {
            if vertex == middle {
                moved.unwrap_or(1_000 + i64::from(vertex) * 2)
            } else {
                1_000 + i64::from(vertex) * 2
            }
        };
        rows.push((
            format!("chain-e-{index}"),
            point(endpoint(index)),
            point(endpoint(index + 1)),
        ));
    }
    for index in 0..unrelated {
        let batch = index / REMOTE_BATCH_EDGES;
        let offset = index % REMOTE_BATCH_EDGES;
        let start = 1_000_000 + i64::from(batch * REMOTE_BATCH_EDGES) * 4 + i64::from(offset) * 2;
        rows.push((
            format!("remote-{batch}-e-{offset}"),
            point(start),
            point(start + 2),
        ));
    }
    rows.sort_by(|a, b| a.0.cmp(&b.0));
    rows
}
fn contacts(directory: &str, vertices: u32, unrelated: u32, moved: Option<i64>) -> Vec<EdgeValue> {
    intersecting(universe(directory, vertices, unrelated, moved), 50)
}
fn intersecting(rows: Vec<EdgeValue>, x: i64) -> Vec<EdgeValue> {
    rows.into_iter()
        .filter(|(_, a, b)| {
            a.lower_x.min(b.lower_x) <= x
                && x <= a.upper_x.max(b.upper_x)
                && a.lower_y.min(b.lower_y) <= 0
                && 0 <= a.upper_y.max(b.upper_y)
        })
        .collect()
}
