//! Genuine small Source populations expose full-tree traversal through actual callbacks.
use super::*;
use crate::application_invariant_acceptance::managed_index::prefix_observation;

#[test]
fn checked_summary_pruning_visits_local_paths_and_preserves_boundary_membership() {
    lifecycle::run(|| {
        for vertices in [33u32, 65] {
            journey(vertices);
        }
    });
}
fn journey(vertices: u32) {
    let calls = Arc::new(provider::Calls::default());
    let config = Configuration::new(calls.clone());
    let prefixes = Arc::new(prefix_observation::Calls::default());
    let world = host::install_growth(
        prefixes.clone(),
        Arc::new(prefix_rule::Calls::new(prefixes)),
        config,
        worth_query_host::facade::admission::authenticated_principal::WorthQueryAdmissionClock::system(),
    );
    let scope = authentication::request_scope();
    let principal = authentication::block_on(
        authentication::admit(world.installed_schema()).authenticate(
            authentication::LocalCredential::issued_for_model_owner(),
            &scope,
        ),
    )
    .unwrap();
    setup(&world, &principal, &scope);
    commit(
        &world,
        &principal,
        &scope,
        Edit::Chain {
            directory: "A".into(),
            shared_directory: None,
            prefix: "local".into(),
            vertices,
            origin: 1_000,
        },
        0x919610 + u64::from(vertices),
    );
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
    assert_eq!(
        source.rows().len(),
        vertices as usize,
        "one original edge and all authored chain edges exist"
    );
    let index = request
        .query(find("A", 1_000))
        .open_index::<provider::BoxIndex>()
        .unwrap();
    // Predetermined bound allows four comparisons per binary level plus three.
    // The actual BVH has 2*SourceCount-1 nodes, so a full walk fails both samples.
    let levels = u64::from(u32::BITS - vertices.leading_zeros());
    let maximum = 4 * levels + 3;
    for x in [1_000i64, 1_002, 999_999] {
        let comparisons = calls.summary_comparisons.load(Ordering::SeqCst);
        let matches = calls.matched_sources.load(Ordering::SeqCst);
        let actual = request.query(find("A", x)).execute().unwrap();
        let expected = (0..vertices - 1)
            .filter(|edge| {
                let start = 1_000 + i64::from(*edge) * 2;
                start <= x && x <= start + 2
            })
            .map(|edge| {
                (
                    format!("local-e-{edge}"),
                    Coordinates::new(
                        1_000 + i64::from(edge) * 2,
                        0,
                        1_000 + i64::from(edge) * 2,
                        0,
                    ),
                    Coordinates::new(
                        1_002 + i64::from(edge) * 2,
                        0,
                        1_002 + i64::from(edge) * 2,
                        0,
                    ),
                )
            })
            .collect::<Vec<_>>();
        assert_eq!(
            values(actual.rows()),
            expected,
            "authored closed-coordinate oracle for {x}"
        );
        let comparisons = calls.summary_comparisons.load(Ordering::SeqCst) - comparisons;
        let matches = calls.matched_sources.load(Ordering::SeqCst) - matches;
        assert!(
            comparisons as u64 <= maximum,
            "vertices={vertices}, x={x}, comparisons={comparisons}, maximum={maximum}"
        );
        assert_eq!(
            matches,
            actual.rows().len(),
            "only surviving genuine leaf predicates run"
        );
        eprintln!(
            "checked summary vertices={vertices} x={x} comparisons={comparisons} matches={matches}"
        );
    }
    index.close();
}
