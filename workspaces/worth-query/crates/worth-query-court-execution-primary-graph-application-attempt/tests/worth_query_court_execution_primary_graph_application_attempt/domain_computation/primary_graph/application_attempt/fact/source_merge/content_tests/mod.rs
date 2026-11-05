#[test]
fn native_comparison_refusals_keep_literal_attempt_categories_and_causes() {
    use worth_query_execution::facade::application_contribution::WorthQueryManagedComputationResourceDenial as Resource;
    use worth_query_execution::facade::application_inspection::StoreDenial;
    use worth_query_execution::facade::domain_computation::WorthQueryApplicationAttemptDenialKind as Kind;
    use worth_query_execution::facade::domain_computation::WorthQueryProviderSessionDenialKind as Session;
    use worth_relational::facade::content::RelationalContentDenial as Denial;
    use worth_relational::facade::content::RelationalContentRejection as Rejection;
    use worth_relational::facade::transactions::RelationalExecutionDenialCause as Cause;
    for (input, cause, kind) in [
            (Denial::Execution(Cause::Cancelled), Cause::Cancelled, Kind::Cancelled),
            (Denial::Execution(Cause::DeadlineElapsed), Cause::DeadlineElapsed, Kind::DeadlineExceeded),
            (Denial::Execution(Cause::WorkExhausted), Cause::WorkExhausted, worth_query_execution::facade::domain_computation::WorthQueryApplicationAttemptDenialKind::ExecutionRequest(Session::ExecutionResource {
                denial: Resource::WorkExhausted, partition_identity: None, policy_ancestor: None,
            })),
            (Denial::ContentIdentityConflict, Cause::Content(Rejection::ContentIdentityConflict), Kind::ContentInconsistency),
            (Denial::SizeLimit { observed: 97, maximum: 31 }, Cause::Content(Rejection::SizeLimit { observed: 97, maximum: 31 }), worth_query_execution::facade::domain_computation::WorthQueryApplicationAttemptDenialKind::ExecutionRequest(Session::Content(Rejection::SizeLimit { observed: 97, maximum: 31 }))),
        ] {
            let actual = StoreDenial::Content(input).into_attempt_denial_for_test("comparison");
            assert_eq!(actual.relational_execution_cause_for_test(), Some(cause));
            assert_eq!(actual.kind(), kind);
        }
}
