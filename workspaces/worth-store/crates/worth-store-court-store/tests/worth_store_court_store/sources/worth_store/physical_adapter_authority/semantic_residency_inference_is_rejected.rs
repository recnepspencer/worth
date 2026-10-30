use worth_store::physical_runtime::PhysicalRecordChunkView;

fn infer_semantic_residency(view: &PhysicalRecordChunkView<'_>) -> bool {
    view.is_semantically_resident()
}

fn main() {}
