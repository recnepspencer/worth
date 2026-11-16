//! Fixed originals reattach their genuine H6 source and output demand after reopen.
use super::{checkpoint_capture, fixture, with_bootstrap_advancement_from_request};
use canonical::{host::*, program};
use worth_query_host::facade::{
    WorthQueryApplicationHome, WorthQueryApplicationHomeLocation, WorthQueryApplicationHomeRequest,
};
use worth_query_testkit_certification_canonical::facade::certification::canonical_transition as canonical;
type Action = ApplicationActionReference<
    CanonicalSchema,
    ApplicationRootComposition,
    program::SourceFeature,
    canonical::edit::EditBinding,
    program::DerivedDelivery,
>;

#[test]
fn directory_fixed_original_resumes_registered_outputs() {
    let path = fixture::location();
    let lease = checkpoint_capture::lease(256 << 20);
    let execution = worth_execution::ExecutionRequest::leased(&lease);
    let home = open(path.clone(), execution);
    let original = {
        let scope = request_scope();
        let principal = authenticate_operator(home.runtime().installed_schema(), &scope);
        let request = home.runtime().request(&principal, &scope);
        let mut value = canonical::model::authored_value();
        value.title = "original fixed edit".into();
        let description = request
            .mutate(canonical::edit::Input {
                identity: "authored-nested".into(),
                value,
            })
            .without_source()
            .idempotency_owned(91_u64)
            .unwrap_or_else(|(_, _, cause)| panic!("capture fixed: {cause:?}"));
        let mut operation = description
            .retain_durable_operation::<Program, Action>(&request, home.runtime())
            .unwrap_or_else(|(_, cause)| panic!("retain fixed: {cause:?}"));
        loop {
            match operation.advance(&request) {
                WorthQueryOperationAdvance::Progress {
                    operation: next,
                    progress,
                } => {
                    operation = next;
                    if matches!(progress, WorthQueryOperationProgress::OutputsStarted) {
                        break;
                    }
                }
                WorthQueryOperationAdvance::Refused { denial, .. }
                | WorthQueryOperationAdvance::Terminal { denial, .. } => {
                    panic!("start original outputs: {denial:?}")
                }
                WorthQueryOperationAdvance::Settled(_) => {
                    panic!("start must expose native output custody before settlement")
                }
            }
        }
        operation
            .handoff_for_reopen(&request)
            .unwrap_or_else(|(_, cause)| panic!("fixed handoff: {cause:?}"))
            .0
    };
    let identity = original.identity();
    home.close(&execution)
        .unwrap_or_else(|cause| panic!("fixed Home close: {cause:?}"))
        .reclaim();
    let home = open(path, execution);
    {
        let scope = request_scope();
        let principal = authenticate_operator(home.runtime().installed_schema(), &scope);
        let request = home.runtime().request(&principal, &scope);
        let description = WorthQueryOriginalOperation::readmit_native::<
            _,
            canonical::edit::Input,
            Program,
            Action,
        >(
            identity,
            &request,
            home.runtime(),
            worth_foundational::facade::ValueDecodeBounds::new(16384, 2048, 256, 64),
        )
        .unwrap_or_else(|cause| panic!("readmit fixed: {cause:?}"));
        let mut operation = description
            .retain_operation::<Program, Action>(home.runtime())
            .unwrap_or_else(|(_, cause)| panic!("reattach fixed: {cause:?}"));
        operation = match operation.close(&request) {
            Err((operation, WorthQueryOperationCloseDenial::NativeObligationRetained)) => operation,
            Err((_, cause)) => panic!("unresolved original close: {cause:?}"),
            Ok(_) => panic!("readmitted output custody must reattach before close"),
        };
        let mut reattached = false;
        let settled = loop {
            match operation.advance(&request) {
                WorthQueryOperationAdvance::Progress {
                    operation: next,
                    progress,
                } => {
                    reattached |= matches!(
                        progress,
                        WorthQueryOperationProgress::RegisteredOutputsRecovered
                    );
                    operation = next;
                }
                WorthQueryOperationAdvance::Settled(settled) => break settled,
                WorthQueryOperationAdvance::Refused { denial, .. }
                | WorthQueryOperationAdvance::Terminal { denial, .. } => {
                    panic!("resume fixed: {denial:?}")
                }
            }
        };
        assert!(
            reattached,
            "reopen must resume the original registered output demand"
        );
        let value = request
            .query(canonical::query::ReadValue {
                identity: "authored-nested".into(),
            })
            .execute()
            .unwrap();
        assert_eq!(value.rows()[0].title, "original fixed edit");
        drop(value);
        settled
            .close(&request)
            .unwrap_or_else(|(_, cause)| panic!("fixed discharge: {cause:?}"));
    }
    home.close(&execution)
        .unwrap_or_else(|cause| panic!("fixed final close: {cause:?}"))
        .reclaim();
}

fn open(
    path: std::path::PathBuf,
    execution: worth_execution::ExecutionRequest<'_, '_>,
) -> WorthQueryApplicationHome<CanonicalSchema, Program> {
    let declaration = CanonicalSchema::declaration().unwrap();
    let request = WorthQueryApplicationHomeRequest::new(
        WorthQueryApplicationHomeLocation::Directory(path),
        program(),
        WorthQueryApplicationProgramRoster::new(),
        declaration,
        ((),),
        ApplicationBuildIdentity::parse(env!("WORTH_IMPLEMENTATION_BUILD_ID")).unwrap(),
        limits(),
        |_, graph, installed| {
            let binding = installed
                .principal_binding(DocumentPrincipalBinding::reference())
                .unwrap();
            graph.bind_principal(
                &binding,
                WorthQueryApplicationPrincipalKey::new("canonical-value-operator").unwrap(),
                1_u64,
                operator_identity(),
                WorthQueryPrincipalMappingStatus::Enabled,
            )?;
            graph.bind_entity(
                WorthQueryApplicationEntitySeed::new(
                    Document::reference(),
                    WorthQueryApplicationEntityKey::new("nested-row").unwrap(),
                )
                .field(
                    DocumentIdentityField::at("DocumentIdentityField"),
                    "authored-nested".to_owned(),
                )
                .field(
                    DocumentValueField::at("DocumentValueField"),
                    canonical::model::authored_value(),
                ),
            )?;
            Ok(())
        },
    );
    with_bootstrap_advancement_from_request(execution, |phase| request.open_in_advancement(&phase))
        .unwrap()
        .unwrap_or_else(|cause| panic!("fixed Directory installation: {cause:?}"))
}
