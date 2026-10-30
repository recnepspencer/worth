use worth_store::physical_runtime::PhysicalRecordChunkView;

fn inspect_validated_physical_payload(view: &PhysicalRecordChunkView<'_>) {
    let basis = view.basis();
    inspect(
        view.bytes(),
        view.logical_range(),
        basis.store_identity(),
        basis.store_generation(),
        basis.record(),
        basis.frame_coordinate(),
    );
}

fn inspect<A, B, C, D, E, F>(_: A, _: B, _: C, _: D, _: E, _: F) {}

fn main() {}
