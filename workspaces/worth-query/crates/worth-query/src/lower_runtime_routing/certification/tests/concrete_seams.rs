use super::*;

#[test]
fn certification_bundle_phase_six_required_seams_are_concrete() {
    let host_request = crate::bridge_host_request::serial_request();
    let resource_request = worth_execution::ExecutionRequest::serial(&host_request);

    let surface =
        super::super::surface::worth_query_lower_runtime_representative_surface(resource_request);

    for seam_key in required_phase_six_concrete_seams() {
        assert_eq!(
            surface.evidence_source_for(*seam_key),
            Some(
                super::super::surface::WorthQueryLowerRuntimeRepresentativeEvidenceSource::RuntimeBackedFixture
            ),
            "required phase six seam {} must remain runtime-backed",
            seam_key.as_str()
        );
    }
}
