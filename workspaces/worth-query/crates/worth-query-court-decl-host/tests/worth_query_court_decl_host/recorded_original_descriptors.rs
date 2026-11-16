//! Public descriptor DATA codecs; no application, authentication or owner is created.
use std::num::NonZeroU64;
use worth_foundational::facade::{DurableWorldOwnerIdentity, HomeIdentity, WorldOwnerSlotIdentity};
use worth_query_host::facade::{
    application_entry::RequiredWorkflowAssessment, primary_graph::WorthQueryOriginalOutputDemand,
};
use worth_relational::facade::identity::{EntityId, PartitionId};

#[test]
fn discovery_decoder_requires_the_current_validated_program_revision() {
    use worth_query_host::facade::application_entry::WorthQueryWorkflowInstanceDiscoveryIntent;
    use worth_query_testkit_certification_canonical::facade::certification::application_graph::canonical_transition::{
        host, schema::CanonicalSchema,
    };

    let program = host::program();
    let installed = *program.revision();
    let subject = EntityId::new(PartitionId::main(), 4, 1);
    let input = rmp_serde::to_vec(&([2_u8; 32], subject, *installed.as_bytes())).unwrap();
    let locator =
        WorthQueryWorkflowInstanceDiscoveryIntent::<CanonicalSchema>::from_recorded_bytes(
            &input, installed,
        )
        .expect("matching validated program revision admits locator DATA");
    let matching = locator.to_recorded_bytes().unwrap();
    assert_eq!(
        WorthQueryWorkflowInstanceDiscoveryIntent::from_recorded_bytes(&matching, installed)
            .unwrap(),
        locator
    );
    let (identity, subject, mut revision): ([u8; 32], EntityId, [u8; 32]) =
        rmp_serde::from_slice(&matching).unwrap();
    revision[0] ^= 1;
    let stale = rmp_serde::to_vec(&(identity, subject, revision)).unwrap();
    let refusal =
        WorthQueryWorkflowInstanceDiscoveryIntent::<CanonicalSchema>::from_recorded_bytes(
            &stale, installed,
        )
        .expect_err("well-formed stale locator revision must be refused by the decoder");
    assert!(refusal
        .to_string()
        .contains("workflow program revision mismatch"));
    assert!(
        WorthQueryWorkflowInstanceDiscoveryIntent::<CanonicalSchema>::from_recorded_bytes(
            &[],
            installed,
        )
        .is_err()
    );
}

#[test]
fn original_demand_roundtrip_preserves_position_and_basis_and_refuses_zero() {
    let observed = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_nanos()
        .to_le_bytes();
    let owner = DurableWorldOwnerIdentity::from_recorded(
        HomeIdentity::from_observed_bytes(observed).unwrap(),
        WorldOwnerSlotIdentity::from_recorded(2),
    );
    let coordinates = (NonZeroU64::new(7).unwrap(), owner, 9_u64);
    let input = rmp_serde::to_vec(&coordinates).unwrap();
    let descriptor = WorthQueryOriginalOutputDemand::from_recorded_bytes(&input).unwrap();
    let encoded = descriptor.to_recorded_bytes().unwrap();
    let after: (NonZeroU64, DurableWorldOwnerIdentity, u64) =
        rmp_serde::from_slice(&encoded).unwrap();
    assert_eq!(
        after, coordinates,
        "all original Home/basis coordinates survive"
    );
    assert_eq!(
        WorthQueryOriginalOutputDemand::from_recorded_bytes(&encoded).unwrap(),
        descriptor
    );
    assert!(
        WorthQueryOriginalOutputDemand::from_recorded_bytes(&input[..input.len() - 1]).is_err()
    );
    let zero = rmp_serde::to_vec(&(0_u64, owner, coordinates.2)).unwrap();
    assert!(WorthQueryOriginalOutputDemand::from_recorded_bytes(&zero).is_err());
}

#[test]
fn required_occurrence_roundtrip_preserves_exact_workflow_binding() {
    let instance = EntityId::new(PartitionId::main(), 4, 1);
    let fields = (
        instance,
        "review/payment",
        "advance",
        3_u64,
        "payment-detail",
        "payment",
        "summary",
        "assessment",
        "proposal",
        "payment-status",
        "current",
    );
    let input = rmp_serde::to_vec(&fields).unwrap();
    let descriptor = RequiredWorkflowAssessment::from_recorded_bytes(&input).unwrap();
    assert_eq!(
        (
            descriptor.instance(),
            descriptor.node_path(),
            descriptor.transition_identity(),
            descriptor.occurrence(),
            descriptor.query(),
            descriptor.parameter_type(),
            descriptor.result_type(),
            descriptor.binding(),
            descriptor.proposal_identity(),
            descriptor.coverage_identity(),
            descriptor.program_revision()
        ),
        fields
    );
    let encoded = descriptor.to_recorded_bytes().unwrap();
    assert_eq!(
        RequiredWorkflowAssessment::from_recorded_bytes(&encoded).unwrap(),
        descriptor
    );
    assert!(RequiredWorkflowAssessment::from_recorded_bytes(&input[..input.len() - 1]).is_err());
}
