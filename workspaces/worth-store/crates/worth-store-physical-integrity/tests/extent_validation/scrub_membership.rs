use super::support::ExtentFixture;
use worth_store_physical_integrity::*;

#[test]
fn chunk_observation_requires_manifest_and_does_not_retain_its_byte_buffer() {
    let fixture = ExtentFixture::new();
    let mut validator = PhysicalIntegrityScrubValidator::new();
    let chunk = fixture.tail_chunk_bytes();
    let window = PhysicalIntegrityScrubWindow::new(
        1,
        fixture.tail_chunk_scope(),
        UntrustedPhysicalArtifact::from_bounded_bytes(&chunk),
    );
    assert!(matches!(
        validator.inspect(window).0.outcome(),
        PhysicalIntegrityObservationOutcome::Rejected(PhysicalIntegrityRejection::Unknown(_))
    ));
    {
        let manifest = fixture.manifest_bytes();
        let inspection = validator.inspect(PhysicalIntegrityScrubWindow::new(
            0,
            fixture.manifest_scope(),
            UntrustedPhysicalArtifact::from_bounded_bytes(&manifest),
        ));
        assert!(matches!(
            inspection.0.outcome(),
            PhysicalIntegrityObservationOutcome::Intact(_)
        ));
    }
    assert!(matches!(
        validator.inspect(window).0.outcome(),
        PhysicalIntegrityObservationOutcome::Intact(_)
    ));
    validator.invalidate(fixture.manifest_scope());
    assert!(matches!(
        validator.inspect(window).0.outcome(),
        PhysicalIntegrityObservationOutcome::Rejected(PhysicalIntegrityRejection::Unknown(_))
    ));
}

#[test]
fn unrelated_extent_manifest_is_missing_context_not_corruption() {
    let first = ExtentFixture::new();
    let second = ExtentFixture {
        extent: super::support::extent_cell(8, 9),
        record: super::support::record(0x33, 8),
        ..first
    };
    let mut validator = PhysicalIntegrityScrubValidator::new();
    let manifest = first.manifest_bytes();
    validator.inspect(PhysicalIntegrityScrubWindow::new(
        0,
        first.manifest_scope(),
        UntrustedPhysicalArtifact::from_bounded_bytes(&manifest),
    ));
    let chunk = second.tail_chunk_bytes();
    let window = PhysicalIntegrityScrubWindow::new(
        1,
        second.tail_chunk_scope(),
        UntrustedPhysicalArtifact::from_bounded_bytes(&chunk),
    );
    assert!(matches!(
        validator.inspect(window).0.outcome(),
        PhysicalIntegrityObservationOutcome::Rejected(PhysicalIntegrityRejection::Unknown(_))
    ));
    let manifest = second.manifest_bytes();
    validator.inspect(PhysicalIntegrityScrubWindow::new(
        2,
        second.manifest_scope(),
        UntrustedPhysicalArtifact::from_bounded_bytes(&manifest),
    ));
    assert!(matches!(
        validator.inspect(window).0.outcome(),
        PhysicalIntegrityObservationOutcome::Intact(_)
    ));
}
