//! Authenticated framing cannot turn contradictory current facts into ownership.
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
    durability::RelationalNativeCheckpoint, publication::AuthoredRetirementRefusal,
};
type S = Model<6>;
#[test]
fn dropped_added_and_repointed_current_routes_refuse_after_real_frame_admission() {
    let host = install_definition::<S, TwoProgram>();
    let scope = request_scope();
    let principal = authenticate_operator(host.installed_schema(), &scope);
    let request = host.runtime().request(&principal, &scope);
    create(&host, &request, 10, vec![20]);
    settle(&host, &request, 10);
    create(&host, &request, 40, vec![]);
    let failure = super::current_fence::denied(&host);
    let Some(AuthoredRetirementRefusal::OwnedPublicationRequiresCascade { publication, owner }) =
        failure.authored_retirement_refusal()
    else {
        panic!("{failure:?}")
    };
    let checkpoint = host
        .with_host_advancement(|phase| {
            host.capture_application_checkpoint(
                &phase,
                WorthQueryCheckpointCapturePolicy::SystemAllocation,
            )
            .unwrap()
        })
        .unwrap();
    let bytes = checkpoint.bytes();
    let magic = b"WORTH-REL-CHK18\n";
    let start = bytes
        .windows(magic.len())
        .position(|part| part == magic)
        .unwrap();
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
    for change in [0, 1, 2, 9] {
        let changed = native
            .corrupt_current_publication_roster_for_test(&execution, publication, change)
            .unwrap();
        let mut frame = bytes.to_vec();
        frame.splice(start..start + length, changed.bytes().iter().copied());
        frame[start - 16..start - 8].copy_from_slice(&(changed.bytes().len() as u64).to_be_bytes());
        let digest = Sha256::digest(&frame[40..]);
        frame[8..40].copy_from_slice(&digest);
        let recovery = try_reopen::<S>(WorthQueryApplicationCheckpoint::from_untrusted_bytes(
            frame.into_boxed_slice(),
        ));
        let recovered = match recovery {
            Ok(host) => host,
            Err(WorthQueryInMemoryApplicationDenial::Graph(denial)) => {
                super::coherent_roster::assert_owned_recovery(&denial, publication, owner, change);
                continue;
            }
            Err(denial) => panic!("{change}: {denial:?}"),
        };
        let scope = request_scope();
        let principal = authenticate_operator(recovered.installed_schema(), &scope);
        let request = recovered.runtime().request(&principal, &scope);
        let outcome = request
            .advanced()
            .mutate(lifecycle::Plain {
                replacement: false,
                scope: 1,
                id: if change == 9 { 10 } else { 101 },
                descendants: vec![],
            })
            .without_source()
            .idempotency(&"corrupt-current-roster".to_owned())
            .execute_in_program(
                &recovered,
                worth_query_host::facade::runtime::ExecutionAllocationPolicy::SystemAllocation,
            )
            .unwrap();
        let WorthQueryApplicationMutationOutcome::Commit(WorthQueryApplicationUncommitted::Denied(
            denial,
        )) = outcome
        else {
            panic!("{change}: {outcome:?}")
        };
        let error = denial
            .authored_retirement_error()
            .expect("typed current ownership refusal");
        assert_eq!(
            error.authored_retirement_refusal(),
            Some(AuthoredRetirementRefusal::CorruptCurrentOwnership { publication }),
            "{change}: {error:?}"
        );
        assert_eq!(
            error.context.affected_records,
            [worth_relational::facade::transactions::RecordRef::Entity(
                owner
            )],
            "{change}: {error:?}"
        );
        for id in [10, 40, 101, 102] {
            assert!(request.query(query::Input { id }).execute().is_ok());
        }
    }
}

#[test]
fn a_pre_roster_store_cannot_be_read_as_empty_current_ownership() {
    let host = install_definition::<S, TwoProgram>();
    let scope = request_scope();
    let principal = authenticate_operator(host.installed_schema(), &scope);
    let request = host.runtime().request(&principal, &scope);
    create(&host, &request, 10, vec![20]);
    settle(&host, &request, 10);
    create(&host, &request, 40, vec![]);
    let checkpoint = host
        .with_host_advancement(|phase| {
            host.capture_application_checkpoint(
                &phase,
                WorthQueryCheckpointCapturePolicy::SystemAllocation,
            )
            .unwrap()
        })
        .unwrap();
    let mut bytes = checkpoint.bytes().to_vec();
    let position = bytes
        .windows(b"WORTH-REL-CHK18\n".len())
        .position(|slice| slice == b"WORTH-REL-CHK18\n")
        .unwrap();
    bytes[position..position + b"WORTH-REL-CHK17\n".len()].copy_from_slice(b"WORTH-REL-CHK17\n");
    let checksum = Sha256::digest(&bytes[40..]);
    bytes[8..40].copy_from_slice(&checksum);
    let schema = S::declaration().unwrap();
    let denied=in_memory_program_from_checkpoint(
        ApplicationProgramAuthoring::<S,TwoProgram>::begin().with_schema(&schema).validated_program().unwrap(),schema,((),),
        ApplicationBuildIdentity::parse("919b0039-0000-0000-0000-000000000002").unwrap(),
        worth_query_testkit_document_retention::facade::document_retention_model::host::host_limits(worth_query_host::facade::admission::authenticated_principal::WorthQueryAdmissionClock::system()),
        WorthQueryApplicationCheckpoint::from_untrusted_bytes(bytes.into_boxed_slice())).err().expect("old stores require an explicit migration");
    let WorthQueryInMemoryApplicationDenial::Graph(denial) = denied else {
        panic!("{denied:?}")
    };
    assert_eq!(
        denial.kind(),
        WorthQueryPrimaryGraphInstallationDenialKind::CheckpointRecoveryRejected
    );
    assert!(
        denial.subject().contains("UnsupportedLegacySemantics"),
        "{denial:?}"
    );
    super::current_fence::denied(&host);
}
