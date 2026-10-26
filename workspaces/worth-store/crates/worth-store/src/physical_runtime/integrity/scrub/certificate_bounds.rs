use super::PhysicalIntegrityScrubTarget as Target;
use crate::physical_runtime::{
    ManagedPhysicalIntegrityScrubRequest as Request, PhysicalIntegrityScrubRequestDenial as Denial,
};
use std::{num::NonZeroU64, time::Duration};
use worth_store_physical_format::{
    store_namespace::{ProposedStoreIdentity, StoreNamespaceIdentityRecord, StoreNamespaceVersion},
    PhysicalArtifactReadTarget, PhysicalCheckpointIdentity, MAX_CHECKPOINT_CERTIFICATE_BYTES,
};
use worth_store_physical_integrity::{PhysicalArtifactScope, PhysicalByteRange};

fn identity() -> PhysicalCheckpointIdentity {
    let store = StoreNamespaceIdentityRecord::new(
        StoreNamespaceVersion::CURRENT,
        ProposedStoreIdentity::from_nonzero_bytes([0x41; 16]).unwrap(),
    )
    .published_identity();
    PhysicalCheckpointIdentity::new(store, NonZeroU64::new(1).unwrap())
}

#[test]
fn certificate_prefix_is_charged_to_window_and_total_acquisition_limits() {
    let identity = identity();
    let footer = PhysicalByteRange::new(200, 204).unwrap();
    let target = Target::checkpoint_footer_with_certificates(
        identity,
        footer,
        PhysicalByteRange::new(100, 100).unwrap(),
    )
    .unwrap();
    assert_eq!(target.scope().byte_range(), footer);
    assert_eq!(target.declared_bytes(), 304);
    assert_eq!(target.media_range().unwrap().offset(), 100);
    assert_eq!(
        Request::new(
            identity.store_identity(),
            [target],
            204,
            304,
            Duration::from_secs(1)
        )
        .unwrap_err(),
        Denial::WindowBoundExceeded
    );
    assert_eq!(
        Request::new(
            identity.store_identity(),
            [target],
            304,
            204,
            Duration::from_secs(1)
        )
        .unwrap_err(),
        Denial::TotalByteBoundExceeded
    );
    let admitted = Request::new(
        identity.store_identity(),
        [target],
        304,
        304,
        Duration::from_secs(1),
    )
    .unwrap();
    assert_eq!(admitted.targets()[0].declared_bytes(), 304);
}

#[test]
fn certificate_context_preserves_contiguity_overlap_and_wire_bounds() {
    let identity = identity();
    let footer = PhysicalByteRange::new(200, 204).unwrap();
    let target = Target::checkpoint_footer_with_certificates(
        identity,
        footer,
        PhysicalByteRange::new(100, 100).unwrap(),
    )
    .unwrap();
    let binding = Target::new(
        PhysicalArtifactReadTarget::Checkpoint(identity),
        PhysicalArtifactScope::checkpoint_binding(
            identity,
            PhysicalByteRange::new(100, 23).unwrap(),
        ),
    )
    .unwrap();
    assert_eq!(
        Request::new(
            identity.store_identity(),
            [binding, target],
            1024,
            1024,
            Duration::from_secs(1)
        )
        .unwrap_err(),
        Denial::DuplicateOrOverlappingTarget
    );
    assert_eq!(
        Target::checkpoint_footer_with_certificates(
            identity,
            footer,
            PhysicalByteRange::new(100, 99).unwrap(),
        )
        .unwrap_err(),
        Denial::TargetScopeMismatch
    );
    let oversized = PhysicalByteRange::new(1, MAX_CHECKPOINT_CERTIFICATE_BYTES + 1).unwrap();
    assert_eq!(
        Target::checkpoint_footer_with_certificates(
            identity,
            PhysicalByteRange::new(oversized.end_exclusive(), 204).unwrap(),
            oversized,
        )
        .unwrap_err(),
        Denial::TargetScopeMismatch
    );
}
