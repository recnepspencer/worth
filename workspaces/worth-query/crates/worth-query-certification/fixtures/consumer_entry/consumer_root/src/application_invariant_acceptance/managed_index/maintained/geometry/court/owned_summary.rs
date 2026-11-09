//! Heap-owned summaries survive Native insert/update/remove and public reopen.
use super::*;

#[test]
fn owned_exact_summaries_preserve_complete_reads_across_edits_and_reopen() {
    let calls = Arc::new(provider::Calls::default());
    let config = Configuration::new(Arc::clone(&calls));
    let prefix_calls = Arc::new(prefix::Calls::default());
    let prefix_rules = Arc::new(prefix_rule::Calls::new(Arc::clone(&prefix_calls)));
    let world = host::install_geometry(prefix_calls, prefix_rules, config);
    let scope = authentication::request_scope();
    let principal = authentication::block_on(
        authentication::admit(world.installed_schema()).authenticate(
            authentication::LocalCredential::issued_for_model_owner(),
            &scope,
        ),
    )
    .unwrap();
    setup(&world, &principal, &scope);
    let request = world
        .runtime()
        .request(&principal, &scope)
        .on_branch(world.current_world());
    let index = request
        .query(all("B"))
        .open_index::<provider::BoxIndex>()
        .unwrap();
    let retained = request.query(all("B")).execute().unwrap();
    let before = values(retained.rows());
    compare_scan(&world, &principal, &scope, 3);
    commit(
        &world,
        &principal,
        &scope,
        Edit::Edge {
            directory: "B".into(),
            identity: "e4".into(),
            start: "v0".into(),
            end: "v1".into(),
        },
        0x9194e0,
    );
    compare_scan(&world, &principal, &scope, 4);
    commit(
        &world,
        &principal,
        &scope,
        Edit::Move {
            directory: "A".into(),
            vertex: "v1".into(),
            x: 20,
            y: 0,
        },
        0x9194e1,
    );
    compare_scan(&world, &principal, &scope, 4);
    commit(
        &world,
        &principal,
        &scope,
        Edit::Unshare {
            directory: "B".into(),
            edge: "e4".into(),
        },
        0x9194e2,
    );
    compare_scan(&world, &principal, &scope, 3);
    assert_eq!(
        values(retained.rows()),
        before,
        "retained output is isolated"
    );
    index.close();
    let reopened = request
        .query(all("B"))
        .open_index::<provider::BoxIndex>()
        .unwrap();
    assert_eq!(reopened.lifecycle(), WorthQueryManagedIndexLifecycle::Ready);
    compare_scan(&world, &principal, &scope, 3);
    reopened.close();
}

fn compare_scan(
    world: &host::Host,
    principal: &Principal,
    scope: &WorthQueryRequestScope,
    count: usize,
) {
    let request = world
        .runtime()
        .request(principal, scope)
        .on_branch(world.current_world());
    let scan = request
        .query(query::ReadSource {
            directory: "B".into(),
        })
        .execute()
        .unwrap();
    assert_eq!(
        scan.rows().len(),
        count,
        "the independently read Source changed"
    );
    for probe in [-1, 0, 10, 15, 20, 50, 70, 100]
        .into_iter()
        .map(|x| Coordinates::new(x, 0, x, 0))
        .chain([all("B").probe])
    {
        // Interpret the full ordinary Source without Box2, nodes or managed summaries.
        let expected: Vec<_> = values(scan.rows())
            .into_iter()
            .filter(|(_, a, b)| {
                a.lower_x.min(b.lower_x) <= probe.upper_x
                    && probe.lower_x <= a.upper_x.max(b.upper_x)
                    && a.lower_y.min(b.lower_y) <= probe.upper_y
                    && probe.lower_y <= a.upper_y.max(b.upper_y)
            })
            .collect();
        let actual = request
            .query(query::FindBoxes {
                directory: "B".into(),
                probe,
            })
            .execute()
            .unwrap();
        assert!(
            actual.result_set_observation().is_some(),
            "complete managed read"
        );
        assert_eq!(values(actual.rows()), expected);
    }
}
