//! An actual full-u64 Native triangle supplies exact typed bounds through Source.
use super::*;
use worth_query_consumer_values::{IntegerBox2, ModelLocalMetre};
#[test]
fn full_u64_scene_projection_derives_exact_metre_model_local_bounds() {
    let world = installation::install(&crate::installed_schema());
    let scope = authentication::request_scope();
    let adapter = authentication::admit(world.application.installed_schema());
    let principal = authentication::block_on(adapter.authenticate(
        authentication::LocalCredential::issued_for_model_owner(),
        &scope,
    ))
    .unwrap();
    let request = world.application.request(&principal, &scope);
    let m = u64::MAX;
    let vertices = [[1, 1], [m, 1], [1, m]]
        .into_iter()
        .enumerate()
        .map(|(i, [x, y])| PlanarVertex {
            body_key: format!("wide-face-{i}"),
            x: length(x),
            y: length(y),
        })
        .collect();
    require_commit(mutate(
        &request,
        &world.application,
        PlanarMutation {
            scope_key: "anchor-a".into(),
            operation: PlanarOperation::CreateCycle(vertices),
        },
        0x919c01,
    ));
    attach(
        &request,
        &world.application,
        "wide-face-0",
        "wide-scene",
        "wide-face",
        0x919c02,
    );
    let rows = faces(&request, "wide-scene");
    assert_eq!(rows.len(), 1);
    assert_eq!(rows[0].points, [[1, 1], [m, 1], [1, m]]);
    let bounds = rows[0].bounds();
    assert_eq!(bounds.coordinates().lower, [1, 1]);
    assert_eq!(bounds.coordinates().upper, [m, m]);
    assert_eq!(IntegerBox2::<ModelLocalMetre>::LENGTH_UNIT, "Metre");
    assert_eq!(
        IntegerBox2::<ModelLocalMetre>::FRAME,
        "worth.frames.model-local.v1"
    );
}
