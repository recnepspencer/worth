use super::super::facts;
use super::{accepted_without_roles, assert_denied, checkpoint_body, checkpoint_from_body};

#[test]
fn current_facts_roundtrip_older_formats_are_refused_and_hostile_lengths_fail_before_allocation() {
    let fact = crate::domain_computation::primary_graph::application_attempt::WorthQueryApplicationObservedFact::SourceEntity {
        entity_id: worth_relational::facade::identity::EntityId::new(
            worth_relational::facade::identity::PartitionId(1), 3, 1,
        ),
    };
    let bytes = facts::encode(std::slice::from_ref(&fact)).unwrap();
    let mut accepted = accepted_without_roles(b"producer", 0);
    accepted.truncate(accepted.len() - 10);
    accepted.extend_from_slice(&facts::WIRE_VERSION.to_be_bytes());
    accepted.extend_from_slice(&(bytes.len() as u64).to_be_bytes());
    accepted.extend_from_slice(&bytes);
    let decoded = checkpoint_from_body(checkpoint_body(1, accepted.clone()))
        .decode()
        .unwrap();
    assert_eq!(
        decoded.accepted_outputs[0].producer_facts.as_deref(),
        Some(bytes.as_slice())
    );
    assert_eq!(facts::decode(&bytes).unwrap().as_ref(), &[fact]);
    for version in 3..=8_u16 {
        let mut older = checkpoint_body(1, accepted.clone());
        older[..2].copy_from_slice(&version.to_be_bytes());
        assert_denied(
            older,
            &format!("Query application checkpoint format {version} is unsupported"),
        );
    }

    let length_start = accepted.len() - bytes.len() - 8;
    accepted[length_start..length_start + 8]
        .copy_from_slice(&((facts::MAXIMUM_FACT_BYTES + 1) as u64).to_be_bytes());
    assert_denied(
        checkpoint_body(1, accepted),
        "producer fact payload length is invalid",
    );
}
