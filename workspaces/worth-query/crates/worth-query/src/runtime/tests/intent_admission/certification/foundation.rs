use super::*;
use std::collections::BTreeSet;

#[test]
fn intent_admission_certification_bundle_assembles_phase_six_foundation_artifacts() {
    let host_request = crate::test_execution_policy::serial_request();
    let request_execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let bundle = certify_intent_admission(request_execution);
    let required_outputs = worth_query_intent_admission_required_certification_outputs();
    let closeout_extension_outputs = worth_query_intent_admission_closeout_extension_outputs();
    let output_manifest = worth_query_intent_admission_certification_output_manifest();
    let output_names = bundle
        .outputs()
        .iter()
        .map(|output| output.name())
        .collect::<Vec<_>>();

    assert_eq!(
        bundle.family_inventory().rows(),
        worth_query_intent_admission_family_inventory().rows()
    );
    assert_eq!(
        bundle.coverage_inventory().rows(),
        worth_query_intent_admission_coverage_inventory().rows()
    );
    assert_eq!(
        bundle.support_matrix().rows(),
        worth_query_intent_admission_support_matrix().rows()
    );
    assert_eq!(bundle.output_manifest(), output_manifest);
    assert_eq!(output_manifest.len(), 49);
    assert_eq!(required_outputs.len(), 40);
    assert_eq!(closeout_extension_outputs.len(), 9);
    assert_eq!(
        output_names.len(),
        output_names.iter().copied().collect::<BTreeSet<_>>().len(),
        "certification output manifest must be duplicate-free"
    );
    assert_eq!(
        required_outputs.len(),
        required_outputs
            .iter()
            .copied()
            .collect::<BTreeSet<_>>()
            .len(),
        "required certification outputs must be duplicate-free"
    );
    assert_eq!(
        closeout_extension_outputs.len(),
        closeout_extension_outputs
            .iter()
            .copied()
            .collect::<BTreeSet<_>>()
            .len(),
        "closeout extension outputs must be duplicate-free"
    );
    assert_eq!(
        required_outputs
            .iter()
            .copied()
            .collect::<BTreeSet<_>>()
            .intersection(
                &closeout_extension_outputs
                    .iter()
                    .copied()
                    .collect::<BTreeSet<_>>()
            )
            .count(),
        0,
        "required certification outputs and closeout extensions must stay disjoint"
    );
    assert_eq!(output_names, output_manifest);
    assert_eq!(
        output_manifest
            .iter()
            .copied()
            .collect::<BTreeSet<_>>(),
        required_outputs
            .iter()
            .chain(closeout_extension_outputs.iter())
            .copied()
            .collect::<BTreeSet<_>>(),
        "final output manifest must be exactly the union of spec-required outputs and explicit closeout extensions"
    );
    assert_eq!(
        bundle.output_digest("decision_phase_progression_digest"),
        Some(
            bundle
                .proof_shape_audit()
                .decision_phase_progression_digest()
        )
    );
    assert_eq!(
        bundle.output_digest("decision_proof_shape_digest"),
        Some(bundle.proof_shape_audit().decision_proof_shape_digest())
    );
    assert_eq!(
        bundle.output_digest("query_digest"),
        bundle
            .representative_output_report()
            .digest_for("query_digest")
    );
    assert_eq!(
        bundle.output_digest("failure_digest"),
        bundle
            .representative_output_report()
            .digest_for("failure_digest")
    );
    assert_eq!(
        bundle.output_digest("decision_oracle_digest"),
        Some(bundle.oracle_report().oracle_digest())
    );
    assert_eq!(
        bundle.output_digest("legacy_delegation_parity_digest"),
        Some(
            bundle
                .legacy_parity_report()
                .legacy_delegation_parity_digest()
        )
    );
    assert_eq!(
        bundle.output_digest("decision_support_traceability_digest"),
        Some(
            bundle
                .support_traceability_report()
                .decision_support_traceability_digest()
        )
    );
    assert_eq!(
        bundle.output_digest("counter_snapshot"),
        Some(bundle.counter_snapshot().digest())
    );
    assert_eq!(
        bundle.output_digest("seeded_sequence_digest"),
        Some(bundle.seeded_report().seeded_sequence_digest())
    );
    assert_eq!(
        bundle.output_digest("seed_replay_digest"),
        Some(bundle.seeded_report().seed_replay_digest())
    );
    assert_eq!(
        bundle.output_digest("seed_generator_class_digest"),
        Some(bundle.seeded_report().seed_generator_class_digest())
    );
    assert_eq!(
        bundle.output_digest("intent_topology_audit_digest"),
        Some(bundle.topology_audit().topology_digest())
    );
    assert_eq!(
        bundle.output_digest("representative_family_coverage_digest"),
        Some(
            bundle
                .representative_family_report()
                .representative_family_coverage_digest()
        )
    );
    assert!(!bundle.certification_bundle_digest().is_empty());
}
