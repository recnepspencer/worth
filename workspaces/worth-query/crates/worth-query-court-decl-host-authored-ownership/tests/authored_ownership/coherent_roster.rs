//! Independently anchored current facts reject coherent drops and row/digest damage.
use super::{
    correction_courts::install_definition,
    coupled::{
        history::try_reopen,
        multi::{create, settle, TwoProgram},
    },
    *,
};
use sha2::{Digest, Sha256};
use worth_execution::{ExecutionRequest, SerialRequest};
use worth_foundational::facade::*;
use worth_query_testkit_document_retention::facade::document_retention_model::operator_identity::*;
use worth_relational::facade::{
    durability::{RelationalNativeCheckpoint, RelationalRecoveryStop},
    publication::AuthoredRetirementRefusal,
    storage::RelationalStorageUnavailableCause,
};
#[test]
fn coherent_whole_roster_and_route_drops_and_both_digest_corruptions_refuse_typed() {
    type S = Model<6>;
    let host = install_definition::<S, TwoProgram>();
    let scope = request_scope();
    let principal = authenticate_operator(host.installed_schema(), &scope);
    let request = host.runtime().request(&principal, &scope);
    create(&host, &request, 10, vec![20]);
    settle(&host, &request, 10);
    let sentinel = create(&host, &request, 40, vec![]);
    let sentinel = sentinel
        .outputs_of::<super::lifecycle::CreatedOutputs>()
        .unwrap()
        .entity::<super::lifecycle::Created<S>>()
        .unwrap()
        .entity_id();
    let actual = super::current_fence::denied(&host);
    let Some(AuthoredRetirementRefusal::OwnedPublicationRequiresCascade { publication, owner }) =
        actual.authored_retirement_refusal()
    else {
        panic!("{actual:?}")
    };
    let lost = host
        .runtime()
        .probe_coherent_ownership_loss_for_test(host.runtime().current_world(), owner, &scope)
        .unwrap()
        .unwrap_err();
    let worth_relational::facade::transactions::TransactionCommitError::Publication {
        error, ..
    } = lost
    else {
        panic!("{lost:?}")
    };
    assert_eq!(
        error.authored_retirement_refusal(),
        Some(AuthoredRetirementRefusal::CorruptCurrentOwnership { publication })
    );
    assert_eq!(
        error.context.affected_records,
        [worth_relational::facade::transactions::RecordRef::Entity(
            owner
        )]
    );
    let unmatched = host
        .runtime()
        .probe_unmatched_ownership_presence_for_test(host.runtime().current_world(), owner, &scope)
        .unwrap()
        .unwrap_err();
    let worth_relational::facade::transactions::TransactionCommitError::Publication {
        error, ..
    } = unmatched
    else {
        panic!("{unmatched:?}")
    };
    assert_eq!(
        error.authored_retirement_refusal(),
        Some(AuthoredRetirementRefusal::CorruptCurrentOwnership { publication })
    );
    assert_eq!(
        error.context.affected_records,
        [worth_relational::facade::transactions::RecordRef::Entity(
            owner
        )]
    );
    let checkpoint = host
        .with_host_advancement(|phase| {
            host.capture_application_checkpoint(
                &phase,
                WorthQueryCheckpointCapturePolicy::SystemAllocation,
            )
            .unwrap()
        })
        .unwrap();
    for change in [3, 4, 6, 7, 8] {
        let frame = corrupt_current(&checkpoint, publication, change);
        let recovery = try_reopen::<S>(frame);
        match recovery {
            Err(WorthQueryInMemoryApplicationDenial::Graph(denial)) => {
                if change == 8 {
                    let WorthQueryPrimaryGraphInstallationDenialKind::ExecutionDenied {
                        kind: WorthQueryProviderSessionDenialKind::StorageUnavailable(failure),
                    } = denial.kind()
                    else {
                        panic!("{denial:?}")
                    };
                    assert_eq!(failure.cause, RelationalStorageUnavailableCause::Damaged);
                    assert_eq!(
                        failure.subject,
                        worth_relational::facade::storage::RelationalStorageSubject::Region(
                            owner.partition_id
                        )
                    );
                } else {
                    assert_owned_recovery(&denial, publication, owner, change);
                }
            }
            Err(denial) => panic!("{change}: unexpected frame/admission failure: {denial:?}"),
            Ok(recovered) => {
                // Recovery preserves cold residency. The first real custody/fence use must
                // load and authenticate the publication's ordinary current region.
                let result = recovered
                    .runtime()
                    .probe_owned_edge_retarget_for_test(
                        recovered.runtime().current_world(),
                        if matches!(change, 6 | 8) {
                            owner
                        } else {
                            sentinel
                        },
                        owner,
                        &scope,
                    )
                    .unwrap();
                if change == 8 {
                    let error = result.expect_err(
                        "malformed shared catalog is damage, never invented publication zero",
                    );
                    let worth_relational::facade::transactions::TransactionCommitError::Execution {
                        denial,
                        ..
                    } = error
                    else {
                        panic!("{error:?}");
                    };
                    assert!(
                        matches!(denial.kind,worth_relational::facade::transactions::CommitExecutionDenialKind::Cause(worth_relational::facade::transactions::RelationalExecutionDenialCause::StorageUnavailable(failure)) if failure.cause==RelationalStorageUnavailableCause::Damaged && failure.subject==worth_relational::facade::storage::RelationalStorageSubject::Region(owner.partition_id)),
                        "{denial:?}"
                    );
                    continue;
                }
                let error = result.expect_err("coherent removal cannot open the fence");
                let worth_relational::facade::transactions::TransactionCommitError::Publication {
                    error,
                    ..
                } = error
                else {
                    panic!("{change}: {error:?}")
                };
                assert_eq!(
                    error.authored_retirement_refusal(),
                    Some(AuthoredRetirementRefusal::CorruptCurrentOwnership { publication }),
                    "{change}: {error:?}"
                );
                assert_eq!(
                    error.context.affected_records.first(),
                    Some(&worth_relational::facade::transactions::RecordRef::Entity(
                        owner
                    )),
                    "{change}: {error:?}"
                );
            }
        }
    }
}

pub(super) fn corrupt_current(
    checkpoint: &WorthQueryApplicationCheckpoint,
    publication: worth_relational::facade::history::CommitId,
    change: u8,
) -> WorthQueryApplicationCheckpoint {
    let bytes = checkpoint.bytes();
    let magic = b"WORTH-REL-CHK20.1\n";
    let start = bytes.windows(magic.len()).position(|b| b == magic).unwrap();
    let length = u64::from_be_bytes(bytes[start - 16..start - 8].try_into().unwrap()) as usize;
    let native =
        RelationalNativeCheckpoint::from_untrusted_bytes(bytes[start..start + length].to_vec());
    let policy = ExecutionRequestPolicy::new(
        ExecutionPosture::Serial,
        DeterminismContract::CanonicalBitwise,
        ExecutionBudget::new(std::num::NonZeroUsize::MIN, Some(256 * 1024 * 1024), None),
    );
    let payer =
        SerialRequest::from_policy(&policy, worth_execution::CancellationToken::new(), None);
    let execution = ExecutionRequest::serial(&payer);
    let changed = native
        .corrupt_current_publication_roster_for_test(&execution, publication, change)
        .unwrap();
    let mut frame = bytes.to_vec();
    frame.splice(start..start + length, changed.bytes().iter().copied());
    frame[start - 16..start - 8].copy_from_slice(&(changed.bytes().len() as u64).to_be_bytes());
    let digest = Sha256::digest(&frame[40..]);
    frame[8..40].copy_from_slice(&digest);
    WorthQueryApplicationCheckpoint::from_untrusted_bytes(frame.into_boxed_slice())
}

pub(super) fn assert_owned_recovery(
    denial: &WorthQueryPrimaryGraphInstallationDenial,
    publication: worth_relational::facade::history::CommitId,
    owner: worth_relational::facade::identity::EntityId,
    change: u8,
) {
    let Some(RelationalRecoveryStop::Durability(error)) = denial.native_recovery_cause() else {
        panic!("{change}: {denial:?}")
    };
    let Some(failure) = error.storage_unavailable else {
        panic!("{change}: {error:?}")
    };
    assert!(
        matches!(failure.cause,RelationalStorageUnavailableCause::CorruptCurrentOwnership{publication:actual,source,..} if actual==publication && source==owner),
        "{change}: {failure:?}"
    );
}
