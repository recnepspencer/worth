use super::{WorthQueryApplicationAttemptDenial, WorthQueryApplicationAttemptDenialKind};
use crate::domain_computation::primary_graph::resolution_denial::{
    query_read_resolution_denial, WorthQueryPrincipalResolutionDenialKind,
};
use std::error::Error;
use worth_execution::{MapDenial, MapKernelFailure, MapStop, ScanDenial};
use worth_foundational::{
    ExecutionPhysicalReport, ExecutionPosture, ExecutionReport, PartitionIdentity,
};
use worth_relational::facade::runtime::{QueryReadExecutionStop, QueryReadPacketDenial};

#[test]
fn native_query_cause_survives_both_query_denials() {
    for ((attempt_stop, principal_stop), expected) in non_request_causes()
        .into_iter()
        .zip(non_request_causes())
        .zip(non_request_causes())
    {
        assert!(expected.execution_denial().is_none());
        let attempt =
            WorthQueryApplicationAttemptDenial::query_read_denial(attempt_stop, "relation");
        assert_eq!(
            attempt.kind(),
            WorthQueryApplicationAttemptDenialKind::MissingAuthoritativeFact
        );
        assert_eq!(attempt.native_query_read_stop(), Some(&expected));
        assert_eq!(
            attempt
                .source()
                .unwrap()
                .downcast_ref::<QueryReadExecutionStop>(),
            Some(&expected)
        );
        let principal = query_read_resolution_denial(principal_stop, "principal");
        assert_eq!(
            principal.kind(),
            WorthQueryPrincipalResolutionDenialKind::StalePrincipalProof
        );
        assert_eq!(principal.native_query_read_stop(), Some(&expected));
        let copied = principal.clone();
        drop(principal);
        assert_eq!(copied.native_query_read_stop(), Some(&expected));
        assert_eq!(
            copied
                .source()
                .unwrap()
                .downcast_ref::<QueryReadExecutionStop>(),
            Some(&expected)
        );
    }
}

fn non_request_causes() -> Vec<QueryReadExecutionStop> {
    let first = PartitionIdentity::new(1);
    let second = PartitionIdentity::new(2);
    vec![
        QueryReadExecutionStop::PreparationAdmission(ScanDenial::IdentitiesNotCanonical),
        QueryReadExecutionStop::CompletionAdmission(ScanDenial::CoverageMismatch),
        QueryReadExecutionStop::PacketAdmission(MapDenial::CoverageMismatch),
        QueryReadExecutionStop::PacketAdmission(MapDenial::ReadKeysNotCanonical {
            partition: 0,
            earlier: 0,
            later: 1,
        }),
        QueryReadExecutionStop::PacketAdmission(MapDenial::WriteKeysNotCanonical {
            partition: first,
            earlier: 0,
            later: 1,
        }),
        QueryReadExecutionStop::PacketAdmission(MapDenial::WriteSetOverlap {
            earlier: first,
            later: second,
            later_key: 0,
        }),
        QueryReadExecutionStop::PacketAdmission(MapDenial::ReadWriteConflict {
            reader: first,
            writer: second,
            read_key: 0,
        }),
        QueryReadExecutionStop::PacketStopped {
            boundary: Some(first),
            reason: MapStop::Failure {
                identity: first,
                cause: MapKernelFailure::Domain(QueryReadPacketDenial::FragmentUnavailable),
            },
            report: ExecutionReport::new(
                ExecutionPosture::Serial,
                0,
                0,
                ExecutionPhysicalReport::default(),
            ),
        },
    ]
}

#[test]
fn noncanonical_expected_identities_are_one_execution_cause_on_both_query_paths() {
    use crate::domain_computation::WorthQueryProviderSessionDenialKind as Session;
    use worth_relational::facade::transactions::{
        CommitExecutionDenialKind, RelationalExecutionDenialCause as Cause,
    };
    let stop =
        || QueryReadExecutionStop::PacketAdmission(MapDenial::ExpectedIdentitiesNotCanonical);
    assert_eq!(
        stop().execution_denial().unwrap().kind,
        CommitExecutionDenialKind::Cause(Cause::ExpectedIdentitiesNotCanonical)
    );
    let expected = Session::ExecutionIdentitiesNotCanonical {
        partition_identity: None,
    };
    let attempt = WorthQueryApplicationAttemptDenial::query_read_denial(stop(), "relation");
    assert_eq!(
        attempt.kind(),
        WorthQueryApplicationAttemptDenialKind::ExecutionRequest(expected)
    );
    assert_eq!(attempt.native_query_read_stop(), Some(&stop()));
    let principal = query_read_resolution_denial(stop(), "principal");
    assert_eq!(
        principal.kind(),
        WorthQueryPrincipalResolutionDenialKind::ExecutionRequest(expected)
    );
    assert_eq!(principal.native_query_read_stop(), Some(&stop()));
}
