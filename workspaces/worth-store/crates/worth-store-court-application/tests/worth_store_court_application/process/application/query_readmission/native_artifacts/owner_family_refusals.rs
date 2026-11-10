//! Every unsupported or malformed owner family retains its exact kind and record.
use super::*;
#[test]
fn latest_native_owner_rows_refuse_their_actual_family_kind_and_record_before_service() {
    for (name, family) in [
        ("worth-query-workflow-lineage", "workflow"),
        ("worth-query-dispatch-outbox", "outbox"),
        ("worth-query-inbound-completion", "inbound"),
        ("worth-query-aftermath-causality", "aftermath"),
    ] {
        let path = tempfile::tempdir().unwrap();
        let lease = checkpoint_capture::lease(128 * 1024 * 1024);
        let execution = ExecutionRequest::leased(&lease);
        if family != "workflow" {
            fresh_closed(path.path(), execution);
        }
        let (kind, record) = with_bootstrap_advancement_from_request(execution, |phase| {
            if family == "workflow" {
                return corrections::co_published_required_row(path.path(), &phase, name);
            }
            with_native(path.path(), &phase, |native| {
                administration::create_required_row(native, &execution, name)
            })
        })
        .unwrap();
        let refused = reopen_refusal(path.path(), execution);
        match &refused {
            WorthQueryApplicationHomeOpenRefused::ExistingNativeFinalization { query, .. }
                if family == "outbox" || family == "inbound" =>
            {
                assert_eq!(
                    query.reason().subject(),
                    format!(
                        "Home unsupported Native {family} entity kind {} record {record:?}",
                        kind.0
                    )
                )
            }
            WorthQueryApplicationHomeOpenRefused::CustodyReadmission { cause, .. }
                if family == "aftermath" =>
            {
                use worth_query_execution::facade::primary_graph::QueryHomeCorrectionLineageDenial;
                use worth_query_host::facade::HomeCustodyReadmissionDenial;
                assert!(
                    matches!(cause, HomeCustodyReadmissionDenial::Query(
                    QueryHomeCustodyReadmissionDenial::Correction(
                        QueryHomeCorrectionLineageDenial::Record { entity, kind: observed_kind, cause: worth_query_execution::facade::primary_graph::WorthQueryAftermathCausalityReadDenial::Unavailable }
                    )
                 ) if *entity == record && *observed_kind == kind),
                    "{cause:?}"
                );
            }
            WorthQueryApplicationHomeOpenRefused::CustodyReadmission { cause, .. }
                if family == "workflow" =>
            {
                use worth_query_host::facade::HomeCustodyReadmissionDenial;
                assert!(
                    matches!(cause, HomeCustodyReadmissionDenial::Query(
                    QueryHomeCustodyReadmissionDenial::Activation(QueryHomeProgramActivationDenial::Workflow(
                        QueryHomeWorkflowReadmissionDenial::CorruptRecord { entity, kind: observed_kind }
                    ))
                ) if *entity == record && *observed_kind == kind),
                    "{cause:?}"
                );
            }
            other => panic!("latest Native owner gate must refuse before service: {other:?}"),
        }
        refused
            .close(&execution)
            .unwrap_or_else(|refused| panic!("owned family close: {refused:?}"));
    }
}
