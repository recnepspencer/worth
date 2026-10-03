//! Fact wire-version boundaries inside accepted output checkpoint rows.

use super::{accepted_without_roles, assert_denied, checkpoint_body, checkpoint_from_body};

#[test]
fn v6_complete_facts_roundtrip_and_hostile_lengths_fail_before_allocation() {
    let fact = crate::domain_computation::primary_graph::application_attempt::WorthQueryApplicationObservedFact::SourceEntity {
        entity_id: worth_relational::facade::identity::EntityId::new(
            worth_relational::facade::identity::PartitionId(1), 3, 1,
        ),
    };
    let bytes = super::super::facts::encode(std::slice::from_ref(&fact)).unwrap();
    let mut accepted = accepted_without_roles(b"producer", 0);
    accepted.remove(8 + b"producer".len());
    accepted.truncate(accepted.len() - 8);
    accepted.truncate(accepted.len() - 2);
    accepted.extend_from_slice(&6_u16.to_be_bytes());
    accepted.extend_from_slice(&(bytes.len() as u64).to_be_bytes());
    accepted.extend_from_slice(&bytes);
    let mut body = checkpoint_body(1, accepted.clone());
    body[..2].copy_from_slice(&6_u16.to_be_bytes());
    let decoded = checkpoint_from_body(body).decode().unwrap();
    assert_eq!(
        decoded.accepted_outputs[0].producer_facts.as_deref(),
        Some(bytes.as_slice())
    );
    assert_eq!(
        decoded.accepted_outputs[0].posture,
        crate::domain_computation::primary_graph::application_output_demand::WorthQueryAcceptedOutputCheckpointPosture::Performed,
    );
    assert_eq!(
        super::super::facts::decode(&bytes).unwrap().as_ref(),
        &[fact]
    );

    let length_start = accepted.len() - bytes.len() - 8;
    accepted[length_start..length_start + 8]
        .copy_from_slice(&((super::super::facts::MAXIMUM_FACT_BYTES + 1) as u64).to_be_bytes());
    let mut body = checkpoint_body(1, accepted);
    body[..2].copy_from_slice(&6_u16.to_be_bytes());
    assert_denied(body, "producer fact payload length is invalid");
}

#[test]
fn v7_stable_posture_roundtrips_and_unknown_posture_is_rejected() {
    use crate::domain_computation::primary_graph::application_output_demand::WorthQueryAcceptedOutputCheckpointPosture;

    let mut stable = accepted_without_roles(b"producer", 0);
    let posture_at = 8 + b"producer".len();
    stable[posture_at] = 1;
    let decoded = checkpoint_from_body(checkpoint_body(1, stable.clone()))
        .decode()
        .expect("v7 stable posture is descriptive and readable");
    assert_eq!(
        decoded.accepted_outputs[0].posture,
        WorthQueryAcceptedOutputCheckpointPosture::StableReused,
    );
    assert!(decoded.accepted_outputs[0].producer_facts.is_none());

    stable[posture_at] = 2;
    assert_denied(
        checkpoint_body(1, stable),
        "accepted output posture is invalid",
    );
}

#[test]
fn v5_absence_does_not_gain_v6_currentness_during_readmission() {
    use worth_foundational::facade::AspectKey;
    use worth_relational::facade::identity::{EntityId, PartitionId};
    let entity_id = EntityId::new(PartitionId(1), 3, 1);
    for (revision, reusable) in [(None, false), (Some(7), true)] {
        let fact = crate::domain_computation::primary_graph::application_attempt::WorthQueryApplicationObservedFact::SourceAspectRevision {
            entity_id,
            aspect: AspectKey::new("test.optional").unwrap(),
            native_revision: revision,
        };
        let bytes = super::super::facts::encode(&[fact]).unwrap();
        let mut accepted = accepted_without_roles(b"producer", 0);
        accepted.remove(8 + b"producer".len());
        accepted.truncate(accepted.len() - 10);
        accepted.extend_from_slice(&(bytes.len() as u64).to_be_bytes());
        accepted.extend_from_slice(&bytes);
        let mut body = checkpoint_body(1, accepted);
        body[..2].copy_from_slice(&5_u16.to_be_bytes());
        let decoded = checkpoint_from_body(body)
            .decode()
            .expect("v5 map readmits");
        assert_eq!(
            decoded.accepted_outputs[0].producer_facts.is_some(),
            reusable
        );
        assert_eq!(
            decoded.accepted_outputs[0].producer_fact_wire_version,
            if reusable { 5 } else { 0 }
        );
    }
}
