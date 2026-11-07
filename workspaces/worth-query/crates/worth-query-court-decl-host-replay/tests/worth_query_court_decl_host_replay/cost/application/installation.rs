use super::*;
use model::{host::seed_host, programs::*, schema::*};
use worth_query_host::facade::{application_installation::*, primary_graph::*};
use worth_query_testkit_document_retention::facade::document_world_resources::{
    world_resources_with_capacity, WorldLifecycleCapacity,
};

pub fn install(d: Dimensions) -> App {
    assert!([2, 4].contains(&d.installed));
    let mut roster = WorthQueryApplicationProgramRoster::new()
        .support(validated_second_program())
        .support(validated_changed_feature_program())
        .support(model::actor_generation::validated_actor_program())
        .support(model::model_lifecycle::validated_model_lifecycle_program());
    if d.installed == 4 {
        roster = roster
            .support(validated_removed_operation_program())
            .support(validated_changed_operation_program());
    }
    let clock = worth_query_host::facade::admission::authenticated_principal::WorthQueryAdmissionClock::system();
    let resources = world_resources_with_capacity(
        4096,
        256 * 1024 * 1024,
        8192,
        WorldLifecycleCapacity {
            live_product_branches: 1024,
            active_observations: 256,
            owner_created_component_custody_records: 4096,
        },
        clock,
    );
    let limits = WorthQueryInMemoryApplicationLimits::new(
        resources,
        worth_query_host::facade::runtime::WorthQueryApplicationQueryResourceProfile::new(
            std::num::NonZeroUsize::new(128).unwrap(),
        ),
        SignalConditionalEvaluationBudget::development(),
    );
    in_memory_rostered_program(
        validated_first_program(),
        roster,
        DocumentRetentionSchema::declaration().unwrap(),
        ((),),
        ApplicationBuildIdentity::parse("11111111-1111-1111-1111-111111111111").unwrap(),
        limits,
        |phase, graph, installed| {
            seed_host(phase, graph, installed)?;
            for i in 2..d.records {
                graph.bind_entity(
                    WorthQueryApplicationEntitySeed::new(
                        model::schema::Document::reference(),
                        WorthQueryApplicationEntityKey::new(format!("scale-document-{i}")).unwrap(),
                    )
                    .field(
                        DocumentIdentityField::at("DocumentIdentityField"),
                        format!("scale-document-{i}"),
                    )
                    .field(DocumentRetentionField::at("DocumentRetentionField"), 7_u64),
                )?;
            }
            Ok(())
        },
    )
    .unwrap()
}
