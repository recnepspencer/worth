//! Fact wire-version boundaries inside accepted output checkpoint rows.

use super::{accepted_without_roles, assert_denied, checkpoint_body, checkpoint_from_body};

use super::super::facts::{self, WIRE_VERSION};
use crate::domain_computation::primary_graph::application_attempt::WorthQueryApplicationObservedFact as Fact;
use crate::domain_computation::primary_graph::application_output_demand::WorthQueryAcceptedOutputCheckpointPosture as Posture;

fn source_entity() -> Fact {
    Fact::SourceEntity {
        entity_id: worth_relational::facade::identity::EntityId::new(
            worth_relational::facade::identity::PartitionId(1),
            3,
            1,
        ),
    }
}

/// One accepted row of a `format` checkpoint whose producer facts are
/// `bytes`, declared at `fact_version` where the format declares one.
fn body_with_facts(format: u16, fact_version: u16, bytes: &[u8]) -> Vec<u8> {
    let mut accepted = accepted_without_roles(b"producer", 0);
    accepted.truncate(accepted.len() - 10);
    if format < 8 {
        // No accepted-output posture before format 8.
        accepted.remove(8 + b"producer".len());
    } else {
        accepted.extend_from_slice(&fact_version.to_be_bytes());
    }
    accepted.extend_from_slice(&(bytes.len() as u64).to_be_bytes());
    accepted.extend_from_slice(bytes);
    let mut body = checkpoint_body(1, accepted);
    body[..2].copy_from_slice(&format.to_be_bytes());
    body
}

#[test]
fn current_facts_roundtrip_and_hostile_lengths_fail_before_allocation() {
    let fact = source_entity();
    let bytes = facts::encode(std::slice::from_ref(&fact)).unwrap();
    let body = body_with_facts(super::super::FORMAT_VERSION, WIRE_VERSION, &bytes);
    let decoded = checkpoint_from_body(body.clone()).decode().unwrap();
    assert_eq!(
        decoded.accepted_outputs[0].producer_facts.as_deref(),
        Some(bytes.as_slice())
    );
    assert_eq!(
        decoded.accepted_outputs[0].producer_fact_wire_version,
        WIRE_VERSION
    );
    assert_eq!(decoded.accepted_outputs[0].posture, Posture::Performed);
    assert_eq!(facts::decode(&bytes).unwrap().as_ref(), &[fact]);

    let mut hostile = body;
    let length_start = hostile.len() - bytes.len() - 8;
    hostile[length_start..length_start + 8]
        .copy_from_slice(&((facts::MAXIMUM_FACT_BYTES + 1) as u64).to_be_bytes());
    assert_denied(hostile, "producer fact payload length is invalid");
}

#[test]
fn current_stable_posture_roundtrips_and_unknown_posture_is_rejected() {
    use crate::domain_computation::primary_graph::application_output_demand::WorthQueryAcceptedOutputCheckpointPosture;

    let mut stable = accepted_without_roles(b"producer", 0);
    let posture_at = 8 + b"producer".len();
    stable[posture_at] = 1;
    let decoded = checkpoint_from_body(checkpoint_body(1, stable.clone()))
        .decode()
        .expect("current stable posture is descriptive and readable");
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

/// No earlier export can claim the current producer-fact exclusion rule.
#[test]
fn facts_of_an_older_wire_version_are_never_read() {
    let bytes = facts::encode(&[source_entity()]).unwrap();
    for version in 3..=8 {
        assert_denied(
            body_with_facts(version, version, &bytes),
            &format!("Query application checkpoint format {version} is unsupported"),
        );
    }
    for fact_version in 5..=8 {
        assert_denied(
            body_with_facts(super::super::FORMAT_VERSION, fact_version, &bytes),
            "checkpoint producer fact wire version is unsupported",
        );
    }
    assert_denied(
        body_with_facts(super::super::FORMAT_VERSION, WIRE_VERSION + 1, &bytes),
        "checkpoint producer fact wire version is unsupported",
    );
}

#[test]
fn a_version_eight_checkpoint_without_own_write_exclusion_is_refused() {
    let bytes = facts::encode(&[source_entity()]).unwrap();
    let checkpoint = checkpoint_from_body(body_with_facts(8, 8, &bytes));
    assert_eq!(
        checkpoint.decode().err().as_deref(),
        Some("Query application checkpoint format 8 is unsupported")
    );
}
