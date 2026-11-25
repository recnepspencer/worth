//! Operational restore requires the real capture owners, independently of bytes.
use super::*;
use sha2::{Digest, Sha256};
use worth_query_host::facade::primary_graph::WorthQueryPrimaryGraphInstallationDenialKind as Kind;

fn capture(
    application: &support::Application,
) -> application_installation::WorthQueryApplicationCheckpoint {
    application
        .with_host_advancement(|phase| {
            application
                .runtime()
                .capture_application_checkpoint(&phase, CapturePolicy::SystemAllocation)
                .unwrap()
        })
        .unwrap()
}
fn binding_policy() -> worth_foundational::ExecutionRequestPolicy {
    worth_foundational::ExecutionRequestPolicy::new(
        worth_foundational::ExecutionPosture::Serial,
        worth_foundational::DeterminismContract::CanonicalBitwise,
        worth_foundational::ExecutionBudget::new(
            std::num::NonZeroUsize::MIN,
            None,
            Some(8_000_000),
        ),
    )
}
/// Readmits transported bytes under the capture origin `origin_of` retained.
/// Consumers cannot build an origin-less checkpoint from bytes at all.
fn readmit(
    bytes: Vec<u8>,
    origin_of: &application_installation::WorthQueryApplicationCheckpoint,
) -> Result<
    application_installation::WorthQueryApplicationCheckpoint,
    worth_query_host::facade::primary_graph::WorthQueryPrimaryGraphInstallationDenial,
> {
    worth_query_host::facade::application_contribution::with_bootstrap_advancement(
        binding_policy(),
        |phase| {
            application_installation::WorthQueryApplicationCheckpoint::readmit_transported(
                bytes,
                origin_of.capture_origin().unwrap(),
                &phase.execution_request(),
            )
        },
    )
    .unwrap()
}
#[test]
fn an_origin_from_another_capture_cannot_authenticate_equal_state_bytes() {
    let _guard = checkpoint_recovery_test_guard();
    let application = install(None);
    let first = capture(&application);
    let second = capture(&application);
    let denial = readmit(first.bytes().to_vec(), &second).unwrap_err();
    assert_eq!(denial.kind(), Kind::InterimCheckpointCaptureOriginMismatch);
}
#[test]
fn retained_capture_origin_authenticates_transport_after_source_owner_drops() {
    let _guard = checkpoint_recovery_test_guard();
    let application = install(None);
    let checkpoint = capture(&application);
    let origin = checkpoint.capture_origin().unwrap();
    let bytes = checkpoint.bytes().to_vec();
    drop(checkpoint);
    drop(application);
    let transported =
        worth_query_host::facade::application_contribution::with_bootstrap_advancement(
            binding_policy(),
            |phase| {
                application_installation::WorthQueryApplicationCheckpoint::readmit_transported(
                    bytes,
                    origin,
                    &phase.execution_request(),
                )
            },
        )
        .unwrap()
        .unwrap();
    let restored = install(Some(transported));
    let second = capture(&restored);
    drop(restored);
    let restored = install(Some(second));
    let (scope, principal) = authenticate(&restored);
    assert_eq!(
        restored
            .request(&principal, &scope)
            .query(PlanarRead {
                body_key: "anchor-a".into()
            })
            .execute()
            .unwrap()
            .rows()[0]
            .y,
        length(1)
    );
}
#[test]
fn predecessor_interim_format_refuses_readmission_before_capture_origin() {
    let _guard = checkpoint_recovery_test_guard();
    let application = install(None);
    let checkpoint = capture(&application);
    let mut bytes = checkpoint.bytes().to_vec();
    bytes[40..42].copy_from_slice(&13_u16.to_be_bytes());
    let checksum = Sha256::digest(&bytes[40..]);
    bytes[8..40].copy_from_slice(&checksum);
    assert_eq!(
        readmit(bytes, &checkpoint).unwrap_err().kind(),
        Kind::UnsupportedCheckpointFormat {
            observed: 13,
            supported: 14,
        },
    );
}
