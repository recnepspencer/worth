//! equivalent declaration authoring paths share the same digest.
use super::*;

#[test]
fn equivalent_declaration_authoring_paths_share_the_same_digest() {
    let host_request = crate::test_execution_policy::serial_request();
    let request_execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let handle = admitted_handle(request_execution, GeometryOperatingContext::collaborative());
    let left = handle
        .declare(SplitEdgeDeclaration::midpoint("edge:42"))
        .expect("midpoint declaration should canonicalize");
    let right = handle
        .declare(SplitEdgeDeclaration::midpoint_builder("edge:42"))
        .expect("equivalent midpoint declaration should canonicalize");

    assert_eq!(left.declaration_family_key(), "split-edge");
    assert_eq!(
        left.declaration_taxonomy(),
        WorthQueryDeclarationFamilyTaxonomy::new(
            WorthQueryDeclarationPrimaryAuthorityFamily::RelationalTruth,
            WorthQuerySignalCompatibilityPosture::Compatible,
            WorthQueryGroupedDeclarationPosture::NeighborhoodCapable,
        )
    );
    assert_eq!(
        left.declaration_primary_authority_family(),
        WorthQueryDeclarationPrimaryAuthorityFamily::RelationalTruth
    );
    assert_eq!(
        left.declaration_signal_compatibility(),
        WorthQuerySignalCompatibilityPosture::Compatible
    );
    assert_eq!(
        left.declaration_grouped_posture(),
        WorthQueryGroupedDeclarationPosture::NeighborhoodCapable
    );
    assert_eq!(
        handle.family_support::<SplitEdgeFamily>().declare_status(),
        WorthQueryDeclarationCapabilityStatus::Admitted
    );
    let _truth = left.relational_truth();
    let _signal = left.signal_compatible();
    let _grouped = left.neighborhood_capable();
    assert_eq!(left.declaration_digest(), right.declaration_digest());
    assert_eq!(
        left.declaration_meaning_digest(),
        right.declaration_meaning_digest()
    );
}
