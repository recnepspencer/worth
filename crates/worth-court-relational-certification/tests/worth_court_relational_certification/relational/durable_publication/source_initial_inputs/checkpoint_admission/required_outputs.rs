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
    let (captured, observed) = captured_outputs(request, &["image-owned-family"]);
    let (committed, id) = observed[0];
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
    assert_eq!(origin["commit"]["commit_id"], committed);
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
    let root = root_projection::root_payload(raw);
    let directory = crate::source_wire_value::read(root);
    let selector = &directory["roots"][0][1];
    assert!(selector["unresolved_prefix"].is_null());
    assert_eq!(selector["root"]["family"], "image-owned-family");
    assert_eq!(selector["root"]["publication"][0], 0);
    assert_eq!(selector["root"]["publication"][1], committed);
    let key = b"publication";
    let offset = root
        .windows(key.len())
        .position(|value| value == key)
        .unwrap()
        + key.len();
    assert_eq!(root[offset], 0x92);
    assert_eq!(root[offset + 1], 0);
    let mut invalid = root.to_vec();
    invalid[offset + 1] = 2;
    assert_eq!(
        parse(&root_projection::replace_root(raw, &invalid), request).unwrap_err(),
        Refusal::InvalidProjectedMeaning,
    );
}
fn captured_outputs(
    request: &ExecutionRequest<'_, '_>,
    families: &[&str],
) -> (CompleteRelationalNativeCheckpoint, Vec<(u64, EntityId)>) {
    let medium = TestDurableMedium::new();
    let mut runtime = durable_runtime(&medium);
    let permission = runtime.begin_fresh_home_initialization(request).unwrap();
    let receipt = runtime
        .prepare_fresh_home_schema_installation(request, permission)
        .unwrap()
        .install_with_managed_sources(request, additions(), vec![], vec![])
        .unwrap();
    let (owner, installed) = receipt.issue_application_publication_owner();
    let mut observed = Vec::new();
    for family in families {
        let mut transaction = begin_main_transaction(&runtime);
        transaction
            .push_batch(batch_create(family), request.allocation_policy())
            .unwrap();
        let target = CreatedEntityRef {
            partition_id: PartitionId::main(),
            kind_id: KindId(1),
            client_key: worth_relational::facade::symbols::ClientKey::raw(*family),
        };
        let publication = RequiredOutputPublication::Birth {
            admission: owner.admit_required_output("image-creator", [7; 32]),
            absence: None,
            meaning: RequiredOutputPublicationMeaning {
                implementation: RequiredOutputProducingImplementation::from_checkpoint(
                    [1; 16], [2; 32],
                )
                .unwrap(),
                family: (*family).into(),
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
        let committed = result.commit.commit_id.0;
        release_test_commit_snapshot(&runtime, &result);
        observed.push((committed, id));
    }
    (capture(&runtime, request), observed)
}

#[test]
fn checkpoint_image_preserves_balanced_selectors_across_distinct_original_publications() {
    let policy = worth_testkit_relational::shared_relational_certification_test_execution_policy::serial_request();
    let request = &ExecutionRequest::serial(&policy);
    let families = ["a-image-family", "b-image-family", "c-image-family"];
    let (captured, observed) = captured_outputs(request, &families);
    let image = captured
        .admission_image()
        .seal_home_cut(&cut(&captured), request)
        .unwrap();
    let directory = crate::source_wire_value::read(root_projection::root_payload(image.as_bytes()));
    let root = directory["roots"]
        .as_array()
        .unwrap()
        .iter()
        .find(|root| root[0]["commit"] == observed[2].0)
        .unwrap();
    let selector = &root[1];
    assert!(selector["unresolved_prefix"].is_null());
    let mut found = Vec::new();
    selector_families(&selector["root"], &mut found);
    assert_eq!(found, families);
    assert_eq!(selector["root"]["family"], families[1]);
    assert_eq!(selector["root"]["left"]["publication"][1], observed[0].0);
    assert_eq!(selector["root"]["publication"][1], observed[1].0);
    assert_eq!(selector["root"]["right"]["publication"][1], observed[2].0);
}

fn selector_families<'a>(node: &'a serde_json::Value, found: &mut Vec<&'a str>) {
    if node.is_null() {
        return;
    }
    selector_families(&node["left"], found);
    found.push(node["family"].as_str().unwrap());
    selector_families(&node["right"], found);
}
