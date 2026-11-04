use std::error::Error;
use worth_execution::MapDenial;
use worth_execution::MapKernelFailure;
use worth_execution::MapStop;
use worth_execution::ScanDenial;
use worth_foundational::ExecutionPhysicalReport;
use worth_foundational::ExecutionPosture;
use worth_foundational::ExecutionReport;
use worth_foundational::PartitionIdentity;
use worth_query_execution::court_doors_for_test::domain_computation::primary_graph::resolution_denial::query_read_resolution_denial;
use worth_query_execution::facade::domain_computation::WorthQueryApplicationAttemptDenial;
use worth_query_execution::facade::domain_computation::WorthQueryApplicationAttemptDenialKind;
use worth_query_execution::facade::domain_computation::WorthQueryPrincipalResolutionDenialKind;
use worth_query_execution::facade::domain_computation::WorthQueryPrincipalResolutionRefusalKind as PrincipalRefusal;
use worth_relational::facade::runtime::QueryReadExecutionStop;
use worth_relational::facade::runtime::QueryReadPacketDenial;

#[test]
fn native_query_cause_survives_both_query_denials() {
    for ((attempt_stop, principal_stop), expected) in non_request_causes()
        .into_iter()
        .zip(non_request_causes())
        .zip(non_request_causes())
    {
        assert!(expected.execution_denial().is_none());
        let attempt = WorthQueryApplicationAttemptDenial::query_read_denial_for_test(
            attempt_stop,
            "relation",
        );
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
            WorthQueryPrincipalResolutionDenialKind::Refused(PrincipalRefusal::StalePrincipalProof)
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

pub fn non_request_causes() -> Vec<QueryReadExecutionStop> {
    let first = PartitionIdentity::new(1);
    let second = PartitionIdentity::new(2);
    vec![
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
    use worth_query_execution::facade::domain_computation::WorthQueryProviderSessionDenialKind as Session;
    use worth_relational::facade::transactions::CommitExecutionDenialKind;
    use worth_relational::facade::transactions::RelationalExecutionDenialCause as Cause;
    let sources: [fn() -> QueryReadExecutionStop; 3] = [
        || QueryReadExecutionStop::PacketAdmission(MapDenial::ExpectedIdentitiesNotCanonical),
        || QueryReadExecutionStop::PreparationAdmission(ScanDenial::IdentitiesNotCanonical),
        || QueryReadExecutionStop::CompletionAdmission(ScanDenial::IdentitiesNotCanonical),
    ];
    for source in sources {
        let stop = source();
        assert_eq!(
            stop.execution_denial().unwrap().kind,
            CommitExecutionDenialKind::Cause(Cause::ExpectedIdentitiesNotCanonical)
        );
        let expected = Session::ExecutionIdentitiesNotCanonical {
            partition_identity: None,
        };
        let attempt =
            WorthQueryApplicationAttemptDenial::query_read_denial_for_test(source(), "relation");
        assert_eq!(
            attempt.kind(),
            WorthQueryApplicationAttemptDenialKind::ExecutionRequest(expected)
        );
        assert_eq!(attempt.native_query_read_stop(), Some(&stop));
        let principal = query_read_resolution_denial(source(), "principal");
        assert_eq!(
            principal.kind(),
            WorthQueryPrincipalResolutionDenialKind::Stopped(Ok(expected))
        );
        assert_eq!(principal.native_query_read_stop(), Some(&stop));
    }
}
