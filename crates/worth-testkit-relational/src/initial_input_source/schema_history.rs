//! Real alternating schema-basis commits with two executable schema carriers.
use super::*;
use worth_relational::facade::{config::*, runtime::*, schema::*};

pub fn schema_transition_history(
    request: &ExecutionRequest<'_, '_>,
    name_bytes: usize,
    rows: usize,
) -> (RelationalRuntime, Arc<TestDurableMedium>) {
    let registry = |version| {
        crate::aspect_storage::AspectSchemaFixture {
            schema_version_id: SchemaVersionId(version),
            relation_kind_name: "s".repeat(name_bytes),
            ..crate::aspect_storage::AspectSchemaFixture::with_default_declared_aspects(
                CascadeDeletePolicy::CascadeDeleteRelations,
            )
        }
        .build_registry()
    };
    let medium = TestDurableMedium::new();
    let mut runtime = RelationalRuntimeApi::builder()
        .profile(RelationalRuntimeProfile::CertificationCore)
        .schema_registry(registry(1))
        .build_durable(medium.attachment())
        .unwrap();
    let permission = runtime.begin_fresh_home_initialization(request).unwrap();
    runtime
        .prepare_fresh_home_schema_installation(request, permission)
        .unwrap()
        .install_with_managed_sources(request, RelationalSchemaRegistry::new(), vec![], vec![])
        .unwrap();
    for row in 0..rows {
        let source = 1 + row as u32 % 2;
        let target = 3 - source;
        let (_, basis) = runtime
            .owner_component_services()
            .basis_port()
            .observe_branch(&runtime.main_branch_identity())
            .unwrap();
        let transition = ProposedSchemaTransition {
            source_schema_id: SchemaId("test".into()),
            source_schema_version_id: SchemaVersionId(source),
            target_schema_id: SchemaId("test".into()),
            target_schema_version_id: SchemaVersionId(target),
            diff_atoms: vec![SchemaDiffAtom::new(
                SchemaElementRef::new(
                    SchemaElementKind::Schema,
                    SchemaId("test".into()),
                    SchemaVersionId(target),
                    None,
                    "published-basis",
                ),
                vec![SchemaStratum::PublicationContract],
                SchemaPublicationImpact::ObservableSurfaceChanged,
                SchemaSubscriberImpact::None,
                HistoricalInterpretationSensitivity::NotSensitive,
                SchemaDiffDetail::FreeText {
                    detail:
                        "publish another declared schema basis with identical executable contracts"
                            .into(),
                    declared_intent: FreeFormSchemaDiffIntent::Additive,
                },
            )],
        };
        let mut transaction = runtime
            .begin_branch_schema_transition(&basis, transition, None, registry(target))
            .unwrap();
        transaction
            .push_batch(
                durable_publication::batch_create(&format!("schema-{row}")),
                request.allocation_policy(),
            )
            .unwrap();
        let outcome = transaction.commit(request, &runtime).unwrap();
        crate::aspect_storage::publications::release_test_commit_snapshot(&runtime, &outcome);
    }
    (runtime, medium)
}
