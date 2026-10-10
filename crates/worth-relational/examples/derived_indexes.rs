mod support;

use worth_relational::facade::{
    history::BranchId,
    indexes::{DerivedIndexBuildRequest, DerivedIndexDefinition, DerivedIndexId, DerivedIndexKind},
    runtime::RelationalRuntimeApi,
};

fn main() {
    let host_request = support::serial_request();
    let request_execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let runtime = RelationalRuntimeApi::builder()
        .schema_registry(support::demo_schema_registry())
        .build();

    let (created, _entity_id) = support::create_entity(&runtime, "indexed");
    let index = runtime.index_authority().register(DerivedIndexDefinition {
        index_id: DerivedIndexId(0),
        name: "entity.name".to_string(),
        kind: DerivedIndexKind::EntityField {
            field_locator: support::aspect_field_locator("name"),
        },
        branch_scoped: true,
    });

    let build = runtime.index_authority().build_for_commit(
        request_execution,
        DerivedIndexBuildRequest {
            source_commit_id: created.commit.commit_id,
            branch_id: BranchId("main".to_string()),
            index_ids: vec![index.index_id],
        },
    );

    let indexes = runtime.index_access();
    let generation = indexes
        .latest_generation(index.index_id, &BranchId("main".to_string()))
        .expect("latest generation");

    println!(
        "built_generations={} failed_indexes={} latest_generation={} source_commit={}",
        build.generations.len(),
        build.failed_indexes.len(),
        generation.generation_id.0,
        generation.source_commit_id.0
    );
}
