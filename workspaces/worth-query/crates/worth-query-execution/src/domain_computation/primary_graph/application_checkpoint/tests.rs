use sha2::{Digest, Sha256};

use super::{
    merge_accepted_outputs, WorthQueryApplicationCheckpoint, BODY_PREFIX_BYTES, CHECKSUM_BYTES,
    FORMAT_VERSION, MAGIC, MAXIMUM_ENTITY_NAME_BYTES, MAXIMUM_PRODUCER_IDENTITY_BYTES,
    MAXIMUM_ROLE_IDENTITY_BYTES,
};

#[test]
fn current_output_replaces_its_recovered_slot_without_checkpoint_growth() {
    let scope = crate::domain_computation::primary_graph::tests::recoverable_commit_support::committed_recoverable_application()
        .principal_scope()
        .scope();
    let checkpoint = |source, partition| {
        crate::domain_computation::primary_graph::application_output_demand::WorthQueryAcceptedOutputCheckpointIdentity {
            producer: "producer".to_owned(),
            source,
            scope,
            source_partition: partition,
            producer_dependency: None,
            idempotency_key: source,
            resources: None,
            roles: Vec::new(),
            producer_facts: None,
        }
    };
    let stale = checkpoint([1; 32], [7; 32]);
    let sibling = checkpoint([2; 32], [8; 32]);
    let current = checkpoint([3; 32], [7; 32]);

    let first = merge_accepted_outputs(vec![current.clone()], vec![stale.clone(), sibling.clone()]);
    let second = merge_accepted_outputs(first.clone(), vec![stale, sibling.clone()]);

    assert_eq!(first, vec![sibling, current]);
    assert_eq!(second, first);
}

#[test]
fn accepted_output_count_is_bounded_before_allocation() {
    let body = checkpoint_body(1, Vec::new());
    assert_denied(body, "accepted-output count exceeds its payload");
}

#[test]
fn producer_identity_length_and_utf8_are_guarded() {
    let mut zero = 0_u64.to_be_bytes().to_vec();
    zero.resize(super::MINIMUM_V5_ACCEPTED_OUTPUT_BYTES, 0);
    assert_denied(
        checkpoint_body(1, zero),
        "producer identity length is invalid",
    );

    let mut oversized = ((MAXIMUM_PRODUCER_IDENTITY_BYTES + 1) as u64)
        .to_be_bytes()
        .to_vec();
    oversized.resize(super::MINIMUM_V5_ACCEPTED_OUTPUT_BYTES, 0);
    assert_denied(
        checkpoint_body(1, oversized),
        "producer identity length is invalid",
    );

    assert_denied(
        checkpoint_body(1, padded(accepted_prefix(&[0xff], 0, [0; 32]))),
        "producer identity is not UTF-8",
    );
}

#[test]
fn dependency_posture_and_absent_padding_are_guarded() {
    assert_denied(
        checkpoint_body(1, padded(accepted_prefix(b"producer", 0, [1; 32]))),
        "absent producer dependency contains data",
    );
    assert_denied(
        checkpoint_body(1, padded(accepted_prefix(b"producer", 7, [0; 32]))),
        "producer dependency posture is invalid",
    );
}

#[test]
fn role_count_is_bounded_before_allocation() {
    let mut accepted = accepted_prefix(b"producer", 0, [0; 32]);
    accepted.extend_from_slice(&2_u64.to_be_bytes());
    accepted.extend_from_slice(&[0; 32]);
    assert_denied(
        checkpoint_body(1, accepted),
        "output-role count exceeds its payload",
    );
}

#[test]
fn role_identity_length_and_utf8_are_guarded() {
    for (role, expected) in [
        (Vec::new(), "output-role identity length is invalid"),
        (
            vec![b'x'; MAXIMUM_ROLE_IDENTITY_BYTES + 1],
            "output-role identity length is invalid",
        ),
        (vec![0xff], "output-role identity is not UTF-8"),
    ] {
        let accepted = accepted_with_raw_role(&role, 0, b"entity", entity_bytes());
        assert_denied(checkpoint_body(1, accepted), expected);
    }
}

#[test]
fn role_posture_is_closed() {
    assert_denied(
        checkpoint_body(
            1,
            accepted_with_raw_role(b"role", 7, b"entity", entity_bytes()),
        ),
        "output-role posture is invalid",
    );
}

#[test]
fn role_entity_name_length_and_utf8_are_guarded() {
    for (entity_name, expected) in [
        (Vec::new(), "output-role entity name length is invalid"),
        (
            vec![b'x'; MAXIMUM_ENTITY_NAME_BYTES + 1],
            "output-role entity name length is invalid",
        ),
        (vec![0xff], "output-role entity name is not UTF-8"),
    ] {
        let accepted = accepted_with_raw_role(b"role", 0, &entity_name, entity_bytes());
        assert_denied(checkpoint_body(1, accepted), expected);
    }
}

#[test]
fn roles_and_accepted_outputs_require_canonical_order() {
    let mut duplicate_roles = accepted_without_roles(b"producer", 2);
    duplicate_roles.extend_from_slice(&role(b"same", b"entity"));
    duplicate_roles.extend_from_slice(&role(b"same", b"entity"));
    duplicate_roles.extend_from_slice(&0_u64.to_be_bytes());
    assert_denied(
        checkpoint_body(1, duplicate_roles),
        "output roles are duplicated or non-canonical",
    );

    let accepted = accepted_with_raw_role(b"role", 0, b"entity", entity_bytes());
    let mut duplicate_outputs = accepted.clone();
    duplicate_outputs.extend_from_slice(&accepted);
    assert_denied(
        checkpoint_body(2, duplicate_outputs),
        "accepted outputs are duplicated or non-canonical",
    );
}

#[test]
fn duplicate_slot_and_copied_fact_payload_cannot_form_two_recovered_candidates() {
    let first = accepted_without_roles(b"producer", 0);
    let mut changed_source = first.clone();
    // Producer length (8) plus the eight-byte name precede source identity.
    changed_source[16] = 2;
    let mut two_slots = first.clone();
    two_slots.extend_from_slice(&changed_source);
    assert_denied(checkpoint_body(2, two_slots), "output slots are duplicated");

    let mut copied_facts = first.clone();
    copied_facts.truncate(copied_facts.len() - 8);
    let fact = super::facts::encode(&[
        crate::domain_computation::primary_graph::application_attempt::WorthQueryApplicationObservedFact::SourceEntity {
            entity_id: worth_relational::facade::identity::EntityId::new(
                worth_relational::facade::identity::PartitionId(1), 3, 1,
            ),
        },
    ]).unwrap();
    copied_facts.extend_from_slice(&(fact.len() as u64).to_be_bytes());
    copied_facts.extend_from_slice(&fact);
    let mut duplicate_identity = first;
    duplicate_identity.extend_from_slice(&copied_facts);
    assert_denied(
        checkpoint_body(2, duplicate_identity),
        "accepted outputs are duplicated or non-canonical",
    );
}

#[test]
fn trailing_and_truncated_payloads_are_rejected() {
    let mut trailing = checkpoint_body(0, Vec::new());
    trailing.push(1);
    assert_denied(trailing, "payload length differs");

    let mut accepted = accepted_without_roles(b"producer", 1);
    accepted.extend_from_slice(&32_u64.to_be_bytes());
    accepted.extend_from_slice(&[b'r'; 25]);
    assert_denied(checkpoint_body(1, accepted), "payload is truncated");
}

fn checkpoint_body(accepted_count: u64, accepted: Vec<u8>) -> Vec<u8> {
    let mut body = Vec::with_capacity(BODY_PREFIX_BYTES + accepted.len());
    body.extend_from_slice(&FORMAT_VERSION.to_be_bytes());
    body.extend_from_slice(&1_u64.to_be_bytes());
    body.extend_from_slice(&0_u64.to_be_bytes());
    body.extend_from_slice(&accepted_count.to_be_bytes());
    body.extend_from_slice(&accepted);
    body
}

fn accepted_prefix(producer: &[u8], dependency_posture: u8, dependency: [u8; 32]) -> Vec<u8> {
    let mut accepted = Vec::new();
    accepted.extend_from_slice(&(producer.len() as u64).to_be_bytes());
    accepted.extend_from_slice(producer);
    accepted.extend_from_slice(&[1; 32]);
    accepted.extend_from_slice(&entity_bytes());
    accepted.extend_from_slice(&[2; 32]);
    accepted.push(dependency_posture);
    accepted.extend_from_slice(&dependency);
    accepted.extend_from_slice(&[3; 32]);
    accepted.extend_from_slice(&[0; 17]);
    accepted
}

#[test]
fn producer_resource_profile_roundtrips_and_legacy_is_unavailable() {
    let mut accepted = accepted_prefix(b"producer", 0, [0; 32]);
    accepted.truncate(accepted.len() - 17);
    super::resources::encode_profile(
        &mut accepted,
        Some(crate::domain_computation::primary_graph::application_contribution::WorthQueryProducerDemandResources::new(4_096, 8_192)),
    );
    accepted.extend_from_slice(&0_u64.to_be_bytes());
    accepted.extend_from_slice(&0_u64.to_be_bytes());
    let decoded = checkpoint_from_body(checkpoint_body(1, accepted))
        .decode()
        .expect("v5 resource evidence decodes");
    let profile = decoded.accepted_outputs[0].resources.unwrap();
    assert_eq!((profile.work(), profile.retained_bytes()), (4_096, 8_192));

    let mut legacy = accepted_prefix(b"producer", 0, [0; 32]);
    legacy.truncate(legacy.len() - 17);
    legacy.extend_from_slice(&0_u64.to_be_bytes());
    let mut body = checkpoint_body(1, legacy);
    body[..2].copy_from_slice(&3_u16.to_be_bytes());
    let decoded = checkpoint_from_body(body)
        .decode()
        .expect("legacy output identity decodes without new resource evidence");
    assert_eq!(decoded.accepted_outputs[0].resources, None);
    assert_eq!(decoded.accepted_outputs[0].producer_facts, None);

    let mut v4 = accepted_prefix(b"producer", 0, [0; 32]);
    v4.extend_from_slice(&0_u64.to_be_bytes());
    let mut body = checkpoint_body(1, v4);
    body[..2].copy_from_slice(&4_u16.to_be_bytes());
    let decoded = checkpoint_from_body(body)
        .decode()
        .expect("v4 remains readable");
    assert_eq!(decoded.accepted_outputs[0].producer_facts, None);
}

#[test]
fn producer_resource_profile_rejects_invalid_posture_and_padding() {
    let mut malformed = accepted_without_roles(b"producer", 0);
    let position = malformed.len() - 8 - 8 - 17;
    malformed[position] = 7;
    assert_denied(
        checkpoint_body(1, malformed.clone()),
        "producer resource profile is invalid",
    );
    malformed[position] = 0;
    malformed[position + 8] = 1;
    assert_denied(
        checkpoint_body(1, malformed),
        "producer resource profile is invalid",
    );
}

#[test]
fn v5_complete_facts_roundtrip_and_hostile_lengths_fail_before_allocation() {
    let fact = crate::domain_computation::primary_graph::application_attempt::WorthQueryApplicationObservedFact::SourceEntity {
        entity_id: worth_relational::facade::identity::EntityId::new(
            worth_relational::facade::identity::PartitionId(1), 3, 1,
        ),
    };
    let bytes = super::facts::encode(&[fact.clone()]).unwrap();
    let mut accepted = accepted_without_roles(b"producer", 0);
    accepted.truncate(accepted.len() - 8);
    accepted.extend_from_slice(&(bytes.len() as u64).to_be_bytes());
    accepted.extend_from_slice(&bytes);
    let decoded = checkpoint_from_body(checkpoint_body(1, accepted.clone()))
        .decode()
        .unwrap();
    assert_eq!(
        decoded.accepted_outputs[0].producer_facts.as_deref(),
        Some(bytes.as_slice())
    );
    assert_eq!(super::facts::decode(&bytes).unwrap().as_ref(), &[fact]);

    let length_start = accepted.len() - bytes.len() - 8;
    accepted[length_start..length_start + 8]
        .copy_from_slice(&((super::facts::MAXIMUM_FACT_BYTES + 1) as u64).to_be_bytes());
    assert_denied(
        checkpoint_body(1, accepted),
        "producer fact payload length is invalid",
    );
}

fn accepted_without_roles(producer: &[u8], role_count: u64) -> Vec<u8> {
    let mut accepted = accepted_prefix(producer, 0, [0; 32]);
    accepted.extend_from_slice(&role_count.to_be_bytes());
    if role_count == 0 {
        accepted.extend_from_slice(&0_u64.to_be_bytes());
    }
    accepted
}

fn accepted_with_raw_role(
    role_identity: &[u8],
    posture: u8,
    entity_name: &[u8],
    entity: [u8; 16],
) -> Vec<u8> {
    let mut accepted = accepted_without_roles(b"producer", 1);
    accepted.extend_from_slice(&(role_identity.len() as u64).to_be_bytes());
    accepted.extend_from_slice(role_identity);
    accepted.push(posture);
    accepted.extend_from_slice(&(entity_name.len() as u64).to_be_bytes());
    accepted.extend_from_slice(entity_name);
    accepted.extend_from_slice(&entity);
    accepted.extend_from_slice(&0_u64.to_be_bytes());
    accepted
}

fn role(identity: &[u8], entity_name: &[u8]) -> Vec<u8> {
    let mut encoded = Vec::new();
    encoded.extend_from_slice(&(identity.len() as u64).to_be_bytes());
    encoded.extend_from_slice(identity);
    encoded.push(0);
    encoded.extend_from_slice(&(entity_name.len() as u64).to_be_bytes());
    encoded.extend_from_slice(entity_name);
    encoded.extend_from_slice(&entity_bytes());
    encoded
}

fn entity_bytes() -> [u8; 16] {
    let mut entity = [0_u8; 16];
    entity[3] = 1;
    entity[11] = 1;
    entity[15] = 1;
    entity
}

fn padded(mut accepted: Vec<u8>) -> Vec<u8> {
    accepted.resize(super::MINIMUM_V5_ACCEPTED_OUTPUT_BYTES, 0);
    accepted
}

fn assert_denied(body: Vec<u8>, expected: &str) {
    let checkpoint = checkpoint_from_body(body);
    let denial = match checkpoint.decode() {
        Err(denial) => denial,
        Ok(_) => panic!("hostile checkpoint payload must fail closed"),
    };
    assert!(
        denial.contains(expected),
        "expected `{expected}` denial, got `{denial}`"
    );
}

fn checkpoint_from_body(body: Vec<u8>) -> WorthQueryApplicationCheckpoint {
    let checksum = Sha256::digest(&body);
    let mut bytes = Vec::with_capacity(MAGIC.len() + CHECKSUM_BYTES + body.len());
    bytes.extend_from_slice(MAGIC);
    bytes.extend_from_slice(&checksum);
    bytes.extend_from_slice(&body);
    WorthQueryApplicationCheckpoint::from_untrusted_bytes(bytes.into_boxed_slice())
}
