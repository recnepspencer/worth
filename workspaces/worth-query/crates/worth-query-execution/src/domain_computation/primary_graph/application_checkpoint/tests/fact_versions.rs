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
    accepted.extend_from_slice(&fact_version.to_be_bytes());
    accepted.extend_from_slice(&(bytes.len() as u64).to_be_bytes());
    accepted.extend_from_slice(bytes);
    let mut body = checkpoint_body(1, accepted);
    body[..2].copy_from_slice(&format.to_be_bytes());
    body
}

#[test]
fn current_facts_roundtrip_and_hostile_lengths_fail_before_allocation() {
    use worth_foundational::facade::{
        AspectFieldLocator, AspectKey, AspectValue, CanonicalFieldPath, FieldKey, LocatorAuthority,
    };
    let source = source_entity();
    let Fact::SourceEntity { entity_id } = source else {
        unreachable!()
    };
    let locator = AspectFieldLocator::new(
        LocatorAuthority::Authoritative,
        AspectKey::new("checkpoint.test").unwrap(),
        CanonicalFieldPath::single(FieldKey::new("x").unwrap()),
    );
    // Exact native values and absence travel as themselves, never as invented
    // source revisions. This is codec carriage, not an actor admission fixture.
    let expected = vec![
        Fact::SourceEntity { entity_id },
        Fact::Field {
            entity_id,
            kind: worth_relational::facade::identity::KindId(14),
            locator: locator.clone(),
            value: AspectValue::UInt64(31),
        },
        Fact::AbsentField {
            entity_id,
            kind: worth_relational::facade::identity::KindId(14),
            locator,
        },
    ];
    let bytes = facts::encode(&expected).unwrap();
    let body = body_with_facts(super::super::FORMAT_VERSION, WIRE_VERSION, &bytes);
    let decoded = checkpoint_from_body(body.clone()).decode().unwrap();
    assert_eq!(
        decoded.accepted_outputs[0]
            .producer_facts
            .as_ref()
            .map(|bytes| bytes.as_slice()),
        Some(bytes.as_slice())
    );
    assert_eq!(
        decoded.accepted_outputs[0].producer_fact_wire_version,
        WIRE_VERSION
    );
    assert_eq!(decoded.accepted_outputs[0].posture, Posture::Performed);
    assert_eq!(
        facts::decode(
            &bytes,
            None,
            worth_execution::ExecutionAllocationPolicy::SystemAllocation
        )
        .unwrap()
        .as_ref(),
        expected.as_slice()
    );

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

/// These are hostile current envelopes, not genuine predecessor artifacts.
#[test]
fn unsupported_fact_wire_refuses_before_interpreting_its_payload() {
    use crate::domain_computation::primary_graph::WorthQueryPrimaryGraphInstallationDenialKind as Kind;
    let mut invalid = facts::encode(&[source_entity()]).unwrap();
    // Keep a valid count and minimum record width, changing only the fact tag.
    invalid[4] = 255;
    for observed in [WIRE_VERSION - 1, WIRE_VERSION + 1] {
        // An invalid fact tag would fail differently if the reader entered it.
        let checkpoint = checkpoint_from_body(body_with_facts(
            super::super::FORMAT_VERSION,
            observed,
            &invalid,
        ));
        let retained = checkpoint.clone();
        let original = checkpoint.bytes().to_vec();
        let denial = checkpoint.decode().err().expect("unsupported inner wire");
        assert_eq!(
            denial.into_installation_denial().kind(),
            Kind::UnsupportedCheckpointProducerFactVersion {
                observed,
                supported: WIRE_VERSION
            }
        );
        assert_eq!(retained.bytes(), original);
    }
    assert_denied(
        body_with_facts(super::super::FORMAT_VERSION, WIRE_VERSION, &invalid),
        "checkpoint producer fact kind is unsupported",
    );
}
