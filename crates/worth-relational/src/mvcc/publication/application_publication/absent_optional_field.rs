use crate::identity::data::{KindId, PartitionId};
use crate::tests::support::*;
use crate::transactions::data::{
    CreateIntent, CreatedEntityRef, EntitySpec, MutationIntent, WorkerIntentBatch,
};
use worth_foundational::facade::{AspectBinding, ContractValidatedAspectValueView};

#[test]
fn absent_optional_codec_cell_does_not_bypass_present_aspect_provenance() {
    let host = crate::test_execution_policy::serial_request();
    let execution = &worth_execution::ExecutionRequest::serial(&host);
    for derived in [false, true] {
        let fixture = AspectSchemaFixture {
            entity_aspects: vec![entity_summary_struct_aspect(
                aspect_key("summary"),
                field_key("summary"),
            )],
            ..AspectSchemaFixture::default()
        };
        let mut runtime = fixture.build_runtime();
        let receipt = runtime
            .prepare_initial_schema_installation()
            .unwrap()
            .install(crate::schema::data::RelationalSchemaRegistry::new())
            .unwrap();
        let (owner, installed) = receipt.issue_application_publication_owner();
        let target = CreatedEntityRef {
            partition_id: PartitionId::main(),
            kind_id: KindId(1),
            client_key: crate::symbols::data::ClientKey::raw("optional-absent"),
        };
        let mut transaction = test_owner_begin_transaction_for_main(&runtime);
        transaction
            .push_batch(
                WorkerIntentBatch::new("optional-absent").push(MutationIntent::Create(
                    CreateIntent::Entity(EntitySpec {
                        partition_id: target.partition_id,
                        kind_id: target.kind_id,
                        client_key: target.client_key.clone(),
                        fields: single_string_aspect_field_patch(
                            aspect_key("summary"),
                            field_key("title"),
                            "present sibling",
                        ),
                    }),
                )),
                worth_execution::ExecutionAllocationPolicy::SystemAllocation,
            )
            .unwrap();
        let proposal = runtime
            .validate_branch_transaction(execution, transaction)
            .unwrap();
        let proposal = if derived {
            proposal
                .bind_required_output_publication(
                    &installed,
                    crate::mvcc::native_birth_fixture(
                        owner.admit_required_output("producer", [4; 32]),
                        vec![target],
                    ),
                )
                .unwrap()
        } else {
            proposal
        };
        let candidate = runtime
            .prepare_validated_proposal(execution, proposal)
            .unwrap();
        let crate::mvcc::RelationalPublicationOutcome::Performed(performed) = runtime
            .publication_port()
            .compare_and_publish(execution, candidate)
        else {
            panic!("native optional-absent fixture must publish");
        };
        let committed = runtime.settle_performed_publication(performed).unwrap();
        let entity = changed_entities(&committed)[0];
        release_test_commit_snapshot(&runtime, &committed);
        let basis = test_owner_main_basis(&runtime).unwrap();
        let partition = basis
            .inner
            .root
            .partition_state(entity.partition_id)
            .unwrap();
        let record = partition.entity_arena.get(&entity).unwrap();
        let value = record
            .extra()
            .authoritative_aspect_state
            .as_ref()
            .unwrap()
            .get(&aspect_key("summary"))
            .unwrap();
        let ContractValidatedAspectValueView::Struct(value) = value.view() else {
            panic!("native struct carrier remains installed");
        };
        assert!(value.get(&field_key("title")).is_some());
        assert!(value.get(&field_key("status")).is_none());
        let loci = [crate::runtime::CodecFieldLocus {
            kind_id: KindId(1),
            aspect_key: aspect_key("summary"),
            field_key: field_key("status"),
            binding: AspectBinding::EntityField {
                field: field_key("summary"),
            },
        }];
        let closure = crate::history::data::derive_codec_retirement_closure(
            execution, &runtime, &basis, &loci,
        );
        if derived {
            assert_eq!(
                closure
                    .expect("genuine whole produced aspect remains reconstructible")
                    .entities
                    .len(),
                1
            );
        } else {
            assert!(closure.is_err(),"an absent changed optional cell cannot leave a protected sibling carrying the old aspect contract current");
        }
    }
}
