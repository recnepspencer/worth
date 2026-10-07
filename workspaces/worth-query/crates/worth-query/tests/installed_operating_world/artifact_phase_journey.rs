use crate::workflow_request::workflow_request;
use std::collections::BTreeMap;
use worth_query::facade::runtime::ExecutionRequest;

use worth_foundational::facade::RetentionDeliveryProfile;
use worth_query::facade::domain;

use super::installed_operation_fixture::{
    artifact_integrated_workspace, bind_artifact_workflow, move_intent, ArtifactNativeObservation,
    ArtifactNativeValues,
};

const ROWS: usize = 32;

#[test]
fn public_workflow_crosses_managed_artifact_native_evidence_and_resource_authority() {
    let (mut workspace, probe) =
        artifact_integrated_workspace("artifact-phase-2-5-journey").unwrap();
    let trace = bind_artifact_workflow(&workspace)
        .admit_workflow_resources(
            crate::suite::installed_operation_fixture::execution_resource_request(),
            &workspace,
        )
        .unwrap()
        .reexecute(
            move_intent("native-integrated"),
            &mut workspace,
            ExecutionRequest::serial(&workflow_request()),
        )
        .unwrap();

    assert_eq!(trace.stage_receipts().len(), 2);
    assert_eq!(probe.allocations(), 1);
    assert_eq!(probe.disposals(), 1);
    assert_eq!(probe.borrow_observations(), 0);
    let observations = probe.take_native_observations();
    assert_eq!(observations.len(), 1);
    let ArtifactNativeObservation::Success(native) = &observations[0] else {
        panic!("integrated journey did not complete native access");
    };
    let ArtifactNativeValues::Candidates(candidates) = native.values() else {
        panic!("integrated journey did not return candidate rows");
    };
    assert_eq!(candidates.len(), ROWS);
    for (row, candidate) in candidates.iter().enumerate() {
        assert_eq!(candidate.id(), 1_000 + row as u64);
        assert_eq!(candidate.score(), 0.25 + row as f64 * 0.5);
    }
    assert_eq!(
        native.evidence().access_bound(),
        &domain::WorthQueryArtifactNativeAccessBound::RowBatch {
            start_row: 0,
            max_rows: ROWS,
        }
    );
    assert_eq!(native.evidence().counters().provider_contacts, 1);
    assert_eq!(native.evidence().counters().values_exposed, ROWS * 2);
    assert_eq!(native.evidence().counters().source_bytes, ROWS * 16);

    let producer = &trace.stage_receipts()[0];
    let consumer = &trace.stage_receipts()[1];
    assert_eq!(
        consumer.predecessor_receipt_identities(),
        &[producer.identity()]
    );
    assert_eq!(consumer.predecessor_proof_count(), 1);
    assert_eq!(
        trace.operation_resource_evidence().identity(),
        consumer.execution_resources().identity()
    );
    assert_ne!(
        trace.operation_resource_evidence().admission_identity(),
        consumer.execution_resources().admission_identity()
    );
    assert!(!trace.operation_resource_evidence().identity().is_empty());
    assert!(!producer.execution_resources().identity().is_empty());
    assert!(!consumer.execution_resources().identity().is_empty());

    let evidence = consumer
        .domain_evidence()
        .expect("the integrated consumer declares governed evidence");
    assert_eq!(
        evidence.binding().run_identity(),
        Some(consumer.run_identity())
    );
    assert_eq!(evidence.binding().stage_identity(), Some("consume"));
    assert_eq!(
        evidence.binding().operation_identity(),
        consumer.operation_identity()
    );
    assert_eq!(
        evidence.binding().basis_identity(),
        consumer.basis_identity()
    );
    assert_eq!(
        evidence.governance().classification(),
        domain::WorthQueryArtifactClassification::Internal
    );
    assert_eq!(
        evidence.governance().retention(),
        RetentionDeliveryProfile::Ephemeral
    );
    let counters = evidence
        .core()
        .counters()
        .iter()
        .map(|counter| {
            (
                counter.schema().name().as_str(),
                (counter.initial(), counter.observed()),
            )
        })
        .collect::<BTreeMap<_, _>>();
    assert_eq!(counters["artifact-bytes"], (0, (ROWS * 16) as u64));
    assert_eq!(counters["artifact-elements"], (0, (ROWS * 2) as u64));
    assert_eq!(counters["artifact-work"], (0, (ROWS * 2 + 1) as u64));
    assert!(evidence.core().decisions().is_empty());
    assert!(evidence.core().candidate_search().is_none());
    assert!(evidence.core().transformation().is_none());
    assert!(matches!(
        evidence.counter_sidecar(),
        domain::WorthQueryAdmittedDomainEvidenceSidecar::NotApplicable
    ));
    assert!(matches!(
        evidence.decision_sidecar(),
        domain::WorthQueryAdmittedDomainEvidenceSidecar::NotApplicable
    ));
}
