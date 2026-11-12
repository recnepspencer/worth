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
fn refused(checkpoint: application_installation::WorthQueryApplicationCheckpoint, expected: Kind) {
    let seeds = std::cell::Cell::new(0);
    let result = support::try_install_program_with_limits::<CheckpointProgram>(
        Some(checkpoint),
        Default::default(),
        support::limits(support::DEFAULT_RETAINED_COMMITS, support::invalidation()),
        |_| seeds.set(seeds.get() + 1),
    );
    let Err(denial) = result else {
        panic!("an unauthenticated interim transport must refuse");
    };
    let application_installation::WorthQueryApplicationPreparationDenial::Graph(denial) =
        denial.as_ref()
    else {
        panic!("expected the typed interim admission refusal: {denial:?}");
    };
    assert_eq!(denial.kind(), expected);
    assert_eq!(
        seeds.get(),
        0,
        "a refused restore cannot run its initializer"
    );
}
#[test]
fn checkpoint_bytes_without_capture_origin_refuse_before_initialization() {
    let _guard = checkpoint_recovery_test_guard();
    let application = install(None);
    let checkpoint = capture(&application);
    refused(
        application_installation::WorthQueryApplicationCheckpoint::from_untrusted_bytes(
            checkpoint.bytes().to_vec(),
        ),
        Kind::InterimCheckpointCaptureOriginMissing,
    );
}
#[test]
fn an_origin_from_another_capture_cannot_authenticate_equal_state_bytes() {
    let _guard = checkpoint_recovery_test_guard();
    let application = install(None);
    let first = capture(&application);
    let second = capture(&application);
    let transported =
        application_installation::WorthQueryApplicationCheckpoint::from_untrusted_bytes(
            first.bytes().to_vec(),
        );
    let denial = worth_query_host::facade::application_contribution::with_bootstrap_advancement(
        binding_policy(),
        |phase| {
            transported
                .with_capture_origin(second.capture_origin().unwrap(), &phase.execution_request())
        },
    )
    .unwrap()
    .unwrap_err();
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
                application_installation::WorthQueryApplicationCheckpoint::from_untrusted_bytes(
                    bytes,
                )
                .with_capture_origin(origin, &phase.execution_request())
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
fn predecessor_interim_format_refuses_before_capture_origin_or_initialization() {
    let _guard = checkpoint_recovery_test_guard();
    let application = install(None);
    let checkpoint = capture(&application);
    let mut bytes = checkpoint.bytes().to_vec();
    bytes[40..42].copy_from_slice(&13_u16.to_be_bytes());
    let checksum = Sha256::digest(&bytes[40..]);
    bytes[8..40].copy_from_slice(&checksum);
    refused(
        application_installation::WorthQueryApplicationCheckpoint::from_untrusted_bytes(bytes),
        Kind::UnsupportedCheckpointFormat {
            observed: 13,
            supported: 14,
        },
    );
}
