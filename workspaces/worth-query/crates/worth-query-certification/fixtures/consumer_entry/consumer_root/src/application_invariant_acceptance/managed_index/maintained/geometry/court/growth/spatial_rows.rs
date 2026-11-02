//! Alternating real rows expose broad lexicographic unions between disjoint bands.
use super::*;
const HIGH: i64 = 1_i64 << 40;
type Authored = (String, Coordinates, Coordinates);
#[test]
fn separated_alternating_rows_keep_checked_queries_sparse_through_cold_and_warm_edits() {
    lifecycle::run(|| {
        for count in [32, 64] {
            rows(count);
        }
    });
}
fn rows(count: u32) {
    let calls = Arc::new(provider::Calls::default());
    let prefixes = Arc::new(prefix::Calls::default());
    let world = host::install_growth(
        prefixes.clone(),
        Arc::new(prefix_rule::Calls::new(prefixes)),
        Configuration::new(calls.clone()),
    );
    let scope = authentication::request_scope();
    let principal = authenticate(&world, &scope);
    setup(&world, &principal, &scope);
    let mut roster = vec![("e1".into(), point(0, 0), point(10, 0))];
    for index in 0..count {
        let x = 1_000 + 4 * i64::from(index);
        let y = if index % 2 == 0 { 0 } else { HIGH };
        for endpoint in 0..2u32 {
            commit(
                &world,
                &principal,
                &scope,
                Edit::Vertex {
                    directory: "A".into(),
                    identity: format!("row-v-{}", 2 * index + endpoint),
                    x: x + 2 * i64::from(endpoint),
                    y,
                },
                0x9196_0000 + u64::from(3 * index + endpoint),
            );
        }
        let identity = format!("row-e-{index:03}");
        commit(
            &world,
            &principal,
            &scope,
            Edit::Edge {
                directory: "A".into(),
                identity: identity.clone(),
                start: format!("row-v-{}", 2 * index),
                end: format!("row-v-{}", 2 * index + 1),
            },
            0x9196_0000 + u64::from(3 * index + 2),
        );
        roster.push((identity, point(x, y), point(x + 2, y)));
    }
    let scope = authentication::request_scope();
    let principal = authenticate(&world, &scope);
    let request = world
        .runtime()
        .request(&principal, &scope)
        .on_branch(world.current_world());
    let source = request
        .query(query::ReadSource {
            directory: "A".into(),
        })
        .execute()
        .unwrap();
    let mut expected_source = roster.clone();
    expected_source.sort_by(|a, b| b.0.cmp(&a.0));
    assert_eq!(
        delivered(source.rows()),
        expected_source,
        "complete real Source census in its declared order"
    );
    let broad = query::FindBoxes {
        directory: "A".into(),
        probe: Coordinates::new(-1, -1, 10_000, HIGH + 1),
    };
    let index = request
        .query(broad.clone())
        .open_index::<provider::BoxIndex>()
        .unwrap();
    request.query(broad.clone()).evict_index(&index).unwrap();
    request.query(broad.clone()).rebuild_index(&index).unwrap();
    let retained = request.query(broad.clone()).execute().unwrap();
    let retained_values = delivered(retained.rows());
    let mut ordered = roster.clone();
    ordered.sort_by(|a, b| a.0.cmp(&b.0));
    assert_eq!(retained_values, ordered);
    let builds = calls.builds.load(Ordering::SeqCst);
    let maximum = 4 * (u32::BITS - count.leading_zeros()) as usize + 3;
    let span = Coordinates::new(1_000, HIGH / 2, 1_000 + 4 * i64::from(count), HIGH / 2);
    for probe in [
        span,
        point(1_000, 0),
        point(1_002, 0),
        point(1_004, HIGH),
        point(999_999, 0),
    ] {
        check(&world, &principal, &scope, &calls, &roster, probe, maximum);
    }
    let updates = calls.updates.load(Ordering::SeqCst);
    commit(
        &world,
        &principal,
        &scope,
        Edit::Move {
            directory: "A".into(),
            vertex: "row-v-0".into(),
            x: 1_000,
            y: HIGH,
        },
        0x9196_1000,
    );
    assert_eq!(
        calls.updates.load(Ordering::SeqCst),
        updates + 1,
        "the one actual incident edge updates A only"
    );
    roster[1].1 = point(1_000, HIGH);
    for probe in [
        span,
        point(1_000, HIGH / 2),
        point(1_002, 0),
        point(999_999, 0),
    ] {
        check(&world, &principal, &scope, &calls, &roster, probe, maximum);
    }
    assert_eq!(
        delivered(retained.rows()),
        retained_values,
        "retained actual old rows preserve their image"
    );
    assert_eq!(
        calls.builds.load(Ordering::SeqCst),
        builds,
        "warm motion and reads cannot hide a cold reconstruction"
    );
    index.close();
}
fn point(x: i64, y: i64) -> Coordinates {
    Coordinates::new(x, y, x, y)
}
fn delivered(rows: &[query::Row]) -> Vec<Authored> {
    rows.iter()
        .map(|row| (row.identity.clone(), row.start, row.end))
        .collect()
}
fn check(
    world: &host::Host,
    principal: &Principal,
    scope: &WorthQueryRequestScope,
    calls: &provider::Calls,
    roster: &[Authored],
    probe: Coordinates,
    maximum: usize,
) {
    let mut expected = roster
        .iter()
        .filter(|(_, a, b)| {
            a.lower_x.min(b.lower_x) <= probe.upper_x
                && probe.lower_x <= a.upper_x.max(b.upper_x)
                && a.lower_y.min(b.lower_y) <= probe.upper_y
                && probe.lower_y <= a.upper_y.max(b.upper_y)
        })
        .cloned()
        .collect::<Vec<_>>();
    expected.sort_by(|a, b| a.0.cmp(&b.0));
    let comparisons = calls.summary_comparisons.load(Ordering::SeqCst);
    let matches = calls.matched_sources.load(Ordering::SeqCst);
    let (actual, work) = invoice(|| {
        world
            .runtime()
            .request(principal, scope)
            .on_branch(world.current_world())
            .query(query::FindBoxes {
                directory: "A".into(),
                probe,
            })
            .execute()
            .unwrap()
    });
    assert_eq!(
        delivered(actual.rows()),
        expected,
        "authored inclusive enclosure oracle and actual Parent order"
    );
    let comparisons = calls.summary_comparisons.load(Ordering::SeqCst) - comparisons;
    let matches = calls.matched_sources.load(Ordering::SeqCst) - matches;
    assert_eq!(matches, actual.rows().len());
    assert!(
        comparisons <= maximum,
        "Source={} probe={probe:?} comparisons={comparisons} maximum={maximum}",
        roster.len()
    );
    eprintln!("alternating rows Source={} probe={probe:?} summary={comparisons} matches={matches} whole original SDK query={work}",roster.len());
}
