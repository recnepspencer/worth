use worth_query::facade::{
    foundation::WorthQueryEntityIdentity,
    runtime::{WorthQueryAspectTouch, WorthQueryAuthoredAspectValue, WorthQueryWorkspace},
};
pub(crate) fn update_measurement(
    measurement: &WorthQueryEntityIdentity,
    workspace: &mut WorthQueryWorkspace,
) {
    workspace
        .with_execution_request_mut(|execution, workspace| {
            workspace
                .update(execution, measurement.clone(), |entity| {
                    entity.set_aspect(
                        WorthQueryAspectTouch::from_authoring_ingress_text("measurement.value")
                            .expect("static measurement aspect"),
                        WorthQueryAuthoredAspectValue::native(
                            worth_foundational::facade::AspectValue::Float32(
                                worth_foundational::facade::CanonicalF32::from_f32(320.0),
                            ),
                        ),
                    )
                })
                .expect("real Query mutation succeeds")
        })
        .expect("workspace host call opens");
}
