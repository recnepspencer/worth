//! Actual public commands carry full-u64 Metre/model-local values to Native.
use super::*;
use worth_query_consumer_values::{PlanarVertex, PositiveLength};

#[test]
fn full_u64_positive_cycles_publish_and_opposite_or_zero_turns_preserve_the_head() {
    let schema = crate::installed_schema();
    let world = installation::install(&schema);
    let scope = authentication::request_scope();
    let adapter = authentication::admit(world.application.installed_schema());
    let principal = authentication::block_on(adapter.authenticate(
        authentication::LocalCredential::issued_for_model_owner(),
        &scope,
    ))
    .unwrap();
    let request = world.application.request(&principal, &scope);
    let m = u64::MAX;
    for (ordinal, points) in [
        [[1, 1], [m, 1], [1, m]],
        [[1, 1], [m - 1, m - 2], [m, m - 1]],
    ]
    .into_iter()
    .enumerate()
    {
        let cycle = vertices(&format!("wide-{ordinal}"), points);
        let before = source_version(&request);
        let result = mutate(
            &request,
            &world.application,
            input(cycle.clone()),
            0x919a00 + ordinal as u64,
        );
        assert!(
            matches!(
                result,
                WorthQueryApplicationMutationOutcome::Committed { .. }
            ),
            "{result:?}"
        );
        assert_ne!(source_version(&request), before);
        for vertex in &cycle {
            assert_eq!(
                read_y(&request, &vertex.body_key),
                PositiveLength::get(&vertex.y)
            );
        }

        let mut negative = points;
        negative.swap(1, 2);
        let before = source_version(&request);
        require_planar_violation(mutate(
            &request,
            &world.application,
            input(vertices(&format!("negative-{ordinal}"), negative)),
            0x919a10 + ordinal as u64,
        ));
        assert_eq!(source_version(&request), before);
        for vertex in &cycle {
            assert_eq!(
                read_y(&request, &vertex.body_key),
                PositiveLength::get(&vertex.y)
            );
        }
    }
    let before = source_version(&request);
    require_planar_violation(mutate(
        &request,
        &world.application,
        input(vertices("wide-zero", [[1, 1], [m - 1, m - 1], [m, m]])),
        0x919a20,
    ));
    assert_eq!(source_version(&request), before);
}

fn vertices(prefix: &str, points: [[u64; 2]; 3]) -> Vec<PlanarVertex> {
    points
        .into_iter()
        .enumerate()
        .map(|(i, [x, y])| PlanarVertex {
            body_key: format!("{prefix}-{i}"),
            x: length(x),
            y: length(y),
        })
        .collect()
}
fn input(vertices: Vec<PlanarVertex>) -> PlanarMutation {
    PlanarMutation {
        scope_key: "anchor-a".into(),
        operation: PlanarOperation::CreateCycle(vertices),
    }
}
