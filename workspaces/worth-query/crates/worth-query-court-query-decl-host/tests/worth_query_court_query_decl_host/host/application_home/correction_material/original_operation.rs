//! The same installed original advances after graceful reopen under fresh authority.
use super::*;
#[test]
fn directory_original_operation_readmits_the_same_identity_and_payload() {
    use worth_query_host::facade::application_entry::{
        WorthQueryOperationConstructionDenial, WorthQueryOriginalOperation,
        WorthQuerySourceOnlyAdvance,
    };
    use worth_query_host::facade::primary_graph::WorthQueryOriginalOperationDenial as Denial;
    let path = fixture::location();
    let lease = checkpoint_capture::lease(256 << 20);
    let execution = worth_execution::ExecutionRequest::leased(&lease);
    let home = open(path.clone(), execution);
    let (original, ownership) = {
        let scope = scope();
        let principal = label::authentication::authenticate_external(
            home.runtime().installed_schema(),
            "alice",
            Duration::from_secs(120),
            &scope,
        );
        let request = home.runtime().request(&principal, &scope);
        let before = request
            .query(label::AccountSummaryParameters)
            .execute()
            .unwrap();
        let source = &before.observed_sources()[0];
        let ownership = (
            source.root_entity_for_test(),
            source.footprint_for_test().entities_for_test().clone(),
            source.footprint_for_test().adjacencies.clone(),
        );
        drop(before);
        let description = request
            .mutate(LabelEdit {
                status: "open".into(),
                label: "retained".into(),
            })
            .without_source()
            .idempotency_owned("original-owner".to_owned())
            .unwrap_or_else(|(_, _, cause)| panic!("capture original: {cause:?}"));
        let description = match description
            .retain_operation::<program::HomeLabelProgram, program::LabelAction>(home.runtime())
        {
            Err((
                description,
                WorthQueryOperationConstructionDenial::Original(
                    Denial::OwnershipMaterialUnavailable,
                ),
            )) => description,
            Err((_, cause)) => panic!("unrecorded original retention: {cause:?}"),
            Ok(_) => panic!("Directory original must record custody before retaining runtime"),
        };
        let operation = description
            .retain_durable_operation::<program::HomeLabelProgram, program::LabelAction>(
                &request,
                home.runtime(),
            )
            .unwrap_or_else(|(_, cause)| panic!("retain original: {cause:?}"));
        let original = operation
            .handoff_for_reopen(&request)
            .unwrap_or_else(|(_, cause)| panic!("live handoff: {cause:?}"))
            .0;
        (original, ownership)
    };
    let identity = original.identity();
    home.close(&execution)
        .unwrap_or_else(|cause| panic!("original handoff close: {cause:?}"))
        .reclaim();
    let home = open(path, execution);
    {
        let scope = scope();
        let principal = label::authentication::authenticate_external(
            home.runtime().installed_schema(),
            "alice",
            Duration::from_secs(120),
            &scope,
        );
        let request = home.runtime().request(&principal, &scope);
        assert!(matches!(
            original.admit(&request),
            Err(Denial::StaleIncarnation)
        ));
        let description = WorthQueryOriginalOperation::readmit_native::<
            _,
            LabelEdit,
            program::HomeLabelProgram,
            program::LabelAction,
        >(
            identity,
            &request,
            home.runtime(),
            worth_foundational::facade::ValueDecodeBounds::new(8192, 1024, 128, 32),
        )
        .unwrap_or_else(|cause| panic!("fresh original readmission: {cause:?}"));
        let operation = description
            .retain_operation::<program::HomeLabelProgram, program::LabelAction>(home.runtime())
            .unwrap_or_else(|(_, cause)| panic!("fresh native original: {cause:?}"));
        drop(edit(&home));
        let completed = match operation.advance(&request) {
            WorthQuerySourceOnlyAdvance::Completed(completed) => completed,
            WorthQuerySourceOnlyAdvance::Refused { denial, .. } => {
                panic!("original advancement: {denial:?}")
            }
            _ => panic!("ordinary original must complete"),
        };
        assert_eq!(completed.original_input().label, "retained");
        completed
            .close(&request)
            .unwrap_or_else(|(_, cause)| panic!("original close: {cause:?}"));
        let rows = request
            .query(label::AccountSummaryParameters)
            .execute()
            .unwrap();
        assert_eq!(rows.rows()[0].label, "retained");
        let source = &rows.observed_sources()[0];
        assert_eq!((source.root_entity_for_test(), source.footprint_for_test().entities_for_test().clone(), source.footprint_for_test().adjacencies.clone()), ownership,
            "the dependency edit and original advance preserve the same native entities and incident selection");
        assert!(matches!(
            WorthQueryOriginalOperation::readmit_native::<
                _,
                LabelEdit,
                program::HomeLabelProgram,
                program::LabelAction,
            >(
                identity,
                &request,
                home.runtime(),
                worth_foundational::facade::ValueDecodeBounds::new(8192, 1024, 128, 32),
            ),
            Err(Denial::OwnershipMaterialUnavailable)
        ));
    }
    home.close(&execution)
        .unwrap_or_else(|cause| panic!("original final close: {cause:?}"))
        .reclaim();
}
