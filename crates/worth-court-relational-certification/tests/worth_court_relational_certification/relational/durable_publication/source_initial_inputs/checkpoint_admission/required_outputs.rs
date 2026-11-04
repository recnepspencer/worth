//! A genuine installed Native publisher supplies the captured origin meaning.
use super::*;
use worth_relational::facade::{
    identity::{EntityId, KindId, PartitionId},
    mvcc::*,
    transactions::*,
};
use worth_testkit_relational::{
    aspect_storage::publications::begin_main_transaction,
    durable_publication::{batch_create, durable_runtime},
};

#[test]
fn checkpoint_image_records_original_required_output_membership_with_logical_basis() {
    let policy = worth_testkit_relational::shared_relational_certification_test_execution_policy::serial_request();
    let request = &ExecutionRequest::serial(&policy);
    let medium = TestDurableMedium::new();
    let mut runtime = durable_runtime(&medium);
    let permission = runtime.begin_fresh_home_initialization(request).unwrap();
    let receipt = runtime
        .prepare_fresh_home_schema_installation(request, permission)
        .unwrap()
        .install_with_managed_sources(request, additions(), vec![], vec![])
        .unwrap();
    let (owner, installed) = receipt.issue_application_publication_owner();
    let mut transaction = begin_main_transaction(&runtime);
    transaction
        .push_batch(
            batch_create("owned-image-output"),
            request.allocation_policy(),
        )
        .unwrap();
    let target = CreatedEntityRef {
        partition_id: PartitionId::main(),
        kind_id: KindId(1),
        client_key: worth_relational::facade::symbols::ClientKey::raw("owned-image-output"),
    };
    let publication = RequiredOutputPublication::Birth {
        admission: owner.admit_required_output("image-creator", [7; 32]),
        absence: None,
        meaning: RequiredOutputPublicationMeaning {
            implementation: RequiredOutputProducingImplementation::from_checkpoint(
                [1; 16], [2; 32],
            )
            .unwrap(),
            family: "image-owned-family".into(),
            source_scope: EntityId::new(PartitionId::main(), 0, 1),
            source_partition: [3; 32],
            source_epoch: [4; 32],
        },
        roles: vec![RequiredOutputRoleTarget {
            role: "output".into(),
            entity_name: "image-output".into(),
            kind: KindId(1),
            posture: RequiredOutputRolePosture::Create,
            target: EntityReference::Created(target.clone()),
        }],
    };
    let proposal = transaction
        .validate(request, &runtime)
        .unwrap()
        .bind_required_output_publication(&installed, publication)
        .unwrap();
    let prepared = runtime
        .prepare_validated_proposal(request, proposal)
        .unwrap();
    let RelationalPublicationOutcome::Performed(performed) = runtime
        .publication_port()
        .compare_and_publish(request, prepared)
    else {
        panic!("the original installed output publisher performs");
    };
    let result = runtime.settle_performed_publication(performed).unwrap();
    let id = result.created_entity(&target).unwrap();
    let committed = result.commit.clone();
    release_test_commit_snapshot(&runtime, &result);
    let captured = capture(&runtime, request);
    let image = captured
        .admission_image()
        .seal_home_cut(&cut(&captured), request)
        .unwrap();
    let raw = image.as_bytes();
    let mut at = 64 + u32::from_le_bytes(raw[12..16].try_into().unwrap()) as usize;
    let _ = references::group(raw, &mut at);
    at += 16;
    let mut outputs = None;
    for tag in 1..=8 {
        let (actual_tag, value) = references::group(raw, &mut at);
        assert_eq!(actual_tag, tag);
        if tag == 8 {
            outputs = Some(crate::source_wire_value::read(value));
        }
    }
    assert_eq!(at, raw.len());
    let outputs = outputs.unwrap();
    references::no_runtime_identity(&outputs);
    let origin = &outputs["origin"];
    assert_eq!(origin["producer"], "image-creator");
    assert_eq!(
        origin["basis"]["reference"]["observation"]["branch"],
        "main"
    );
    assert_eq!(origin["commit"]["commit_id"], committed.commit_id.0);
    assert_eq!(
        origin["outputs"][0],
        crate::source_wire_value::read(&rmp_serde::to_vec_named(&id).unwrap())
    );
    assert_eq!(origin["members"].as_array().unwrap().len(), 1);
    assert_eq!(origin["members"][0]["role"], "output");
    assert_eq!(origin["members"][0]["entity_name"], "image-output");
    assert_eq!(origin["outputs"].as_array().unwrap().len(), 1);
    assert_eq!(origin["members"][0]["id"], origin["outputs"][0]);
    assert!(outputs["retirements"].as_array().unwrap().is_empty());
}
