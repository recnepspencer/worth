use worth_store::physical_runtime::{
    PhysicalRecordChunkView, PhysicalRecordPressureEvidence, PhysicalResidencyRetryPosture,
};

fn observe_stable_read_inputs(
    view: &PhysicalRecordChunkView<'_>,
    pressure: Option<&PhysicalRecordPressureEvidence>,
) {
    let basis = view.basis();
    observe(
        view.bytes(),
        view.logical_range(),
        basis.store_generation(),
        basis.frame_coordinate(),
    );
    if let Some(pressure) = pressure {
        let retry: PhysicalResidencyRetryPosture = pressure.retry_posture();
        observe_pressure(
            pressure.basis(),
            pressure.store_generation(),
            pressure.scope(),
            retry,
        );
    }
}

fn observe<A, B, C, D>(_: A, _: B, _: C, _: D) {}
fn observe_pressure<A, B, C, D>(_: A, _: B, _: C, _: D) {}

fn main() {}
