//! Header refusal precedes current body/native interpretation and preserves backing.
use super::{checkpoint_body, checkpoint_from_body};
use crate::domain_computation::primary_graph::WorthQueryPrimaryGraphInstallationDenialKind as Kind;

fn refusal(body: Vec<u8>, expected: Kind) {
    let checkpoint = checkpoint_from_body(body);
    let retained = checkpoint.clone();
    let original = checkpoint.bytes().to_vec();
    let backing = checkpoint.bytes().as_ptr();
    let denial = checkpoint
        .decode()
        .err()
        .expect("unsupported header refuses");
    assert_eq!(denial.into_installation_denial().kind(), expected);
    assert_eq!(retained.bytes(), original);
    assert_eq!(retained.bytes().as_ptr(), backing);
}

#[test]
fn unsupported_checkpoint_header_refuses_before_malformed_current_body() {
    // Deliberately hostile framing cases, not authentic predecessor fixtures.
    for observed in [
        0,
        9,
        super::super::FORMAT_VERSION - 1,
        super::super::FORMAT_VERSION + 1,
    ] {
        let mut body = checkpoint_body(0, Vec::new());
        body[..2].copy_from_slice(&observed.to_be_bytes());
        body.truncate(2);
        refusal(
            body,
            Kind::UnsupportedCheckpointFormat {
                observed,
                supported: super::super::FORMAT_VERSION,
            },
        );
    }
}

#[test]
fn unsupported_current_workflow_marker_refuses_before_native_bytes() {
    for (facts, instances) in [(1u16, 4u16), (2, 2), (3, 4), (2, 3), (2, 5)] {
        let mut body = checkpoint_body(0, Vec::new());
        body[2..4].copy_from_slice(&facts.to_be_bytes());
        body[4..6].copy_from_slice(&instances.to_be_bytes());
        refusal(
            body,
            Kind::UnsupportedCheckpointWorkflowContract { facts, instances },
        );
    }
}

#[test]
fn malformed_current_header_remains_corruption_not_unsupported() {
    let mut body = checkpoint_body(0, Vec::new());
    body.truncate(2);
    refusal(body, Kind::CheckpointRecoveryRejected);
    let checkpoint = checkpoint_from_body(checkpoint_body(0, Vec::new()));
    let mut corrupt = checkpoint.bytes().to_vec();
    *corrupt.last_mut().unwrap() ^= 1;
    let denial = super::WorthQueryApplicationCheckpoint::from_untrusted_bytes(corrupt)
        .decode()
        .err()
        .expect("checksum must refuse");
    assert_eq!(
        denial.into_installation_denial().kind(),
        Kind::CheckpointRecoveryRejected
    );
}
