//! Retained no-output delivery reads the native original World occurrence.
use super::{fixture::*, host, readback::*, schema::*};
use crate::document_retention_model::operator_identity::{authenticate_operator, request_scope};
use std::num::NonZeroUsize;
use worth_query_host::facade::{
    application_entry::*, declaration::application_program::*, declaration::application_schema::*,
};
pub(super) struct RetainedProgram;
pub(super) struct RetainedDelivery;
impl ApplicationActionDelivery<SourceOnlyRetentionSchema, SetRetentionBinding>
    for RetainedDelivery
{
    type Mode = ApplicationRetainedCommit;
    const IDENTITY: &'static str = "source-only-retention.retained-delivery.v1";
}
pub(super) type Action = ApplicationActionReference<
    SourceOnlyRetentionSchema,
    ApplicationRootComposition,
    DocumentRetentionFeature,
    SetRetentionBinding,
    RetainedDelivery,
>;
pub(super) type Operation = WorthQuerySourceOnlyOperation<
    SourceOnlyRetentionSchema,
    OriginalRetention,
    RetainedProgram,
    Action,
    ApplicationRetainedCommit,
>;
impl ApplicationProgramDefinition<SourceOnlyRetentionSchema> for RetainedProgram {
    type Contributions = (SourceOnlyRetentionContribution,);
    type Outputs = ApplicationProgramOutputs<ApplicationNoOutputGraph>;
    type Rules = ApplicationRuleLeaf;
    const IDENTITY: ApplicationProgramIdentity =
        ApplicationProgramIdentity::new("source-only-retention.retained-program.v1");
    fn feature_specs() -> Vec<ApplicationFeatureSpec> {
        vec![
            ApplicationFeatureSpec::root::<SourceOnlyRetentionSchema, DocumentRetentionFeature>()
                .action::<SetRetentionBinding, RetainedDelivery>()
                .finish_action()
                .finish(),
        ]
    }
}
pub(super) fn operation(
    host: &host::Host<RetainedProgram>,
    request: &WorthQueryApplicationRequest<'_, '_, '_, SourceOnlyRetentionSchema>,
    days: u64,
    key: u64,
) -> Operation {
    request
        .mutate(OriginalRetention {
            input: SetRetentionInput {
                identity: DOCUMENT_IDENTITY.to_owned(),
                retention_days: days,
            },
        })
        .without_source()
        .preconditions(TypedMutationPreconditions::new().expect_fact(
            DocumentIdentityField::reference(),
            ApplicationEncodedScalarValue::try_new(DOCUMENT_IDENTITY.to_owned()).unwrap(),
        ))
        .idempotency_owned(key)
        .unwrap_or_else(|(_, _, denial)| panic!("input capture: {denial:?}"))
        .retain_operation::<RetainedProgram, Action>(host)
        .unwrap_or_else(|(_, denial)| panic!("retained declaration: {denial:?}"))
}
fn observation(
    evidence: &WorthQuerySourceCompletion<DocumentRetentionWritten>,
) -> &WorthQueryApplicationReadObservation {
    match evidence {
        WorthQuerySourceCompletion::Fresh {
            retained: Some(read),
            ..
        }
        | WorthQuerySourceCompletion::Recovered {
            retained: Some(read),
            ..
        } => read,
        _ => panic!("retained completion requires native observation"),
    }
}
#[test]
fn lost_reply_bounded_retry_reads_original_commit_despite_new_current_truth() {
    let host = host::install_program::<RetainedProgram>();
    let scope = request_scope();
    let principal = authenticate_operator(host.installed_schema(), &scope);
    let (original_commit, replay) = {
        let request = host.runtime().request(&principal, &scope);
        let first = match operation(&host, &request, 18, 5018).advance(&request, None) {
            WorthQuerySourceOnlyAdvance::Completed(done) => done,
            _ => panic!("fresh retained commit"),
        };
        let original_commit = observation(first.evidence()).selected_commit().clone();
        first
            .close(&request)
            .unwrap_or_else(|(_, cause)| panic!("first close: {cause:?}"));
        let later = match operation(&host, &request, 25, 5025).advance(&request, None) {
            WorthQuerySourceOnlyAdvance::Completed(done) => done,
            _ => panic!("later source commit"),
        };
        later
            .close(&request)
            .unwrap_or_else(|(_, cause)| panic!("later close: {cause:?}"));
        assert_eq!(read_retention(host.runtime(), host.current_world()), 25);
        let replay = match operation(&host, &request, 18, 5018).advance(&request, None) {
            WorthQuerySourceOnlyAdvance::Progress(owner) => owner,
            _ => panic!("replay owns actual recovered evidence"),
        };
        assert_eq!(
            replay.posture(),
            WorthQuerySourceOnlyPosture::RetainedDeliveryPending
        );
        let replay = match replay.advance(&request, Some(NonZeroUsize::new(1).unwrap())) {
            WorthQuerySourceOnlyAdvance::Refused {
                operation,
                denial:
                    WorthQuerySourceOnlyDenial::History(
                        WorthQueryApplicationHistorySelectionDenial::CommitUnavailable,
                    ),
            } => operation,
            _ => panic!("bounded miss retains original evidence"),
        };
        (original_commit, replay)
    };
    drop(principal);
    drop(scope);
    let fresh_scope = request_scope();
    let fresh_principal = authenticate_operator(host.installed_schema(), &fresh_scope);
    let fresh = host.runtime().request(&fresh_principal, &fresh_scope);
    let completed = match replay.advance(&fresh, None) {
        WorthQuerySourceOnlyAdvance::Completed(done) => done,
        _ => panic!("funded fresh retained replay"),
    };
    assert_eq!(
        observation(completed.evidence()).selected_commit(),
        &original_commit
    );
    let historical = fresh
        .at(observation(completed.evidence()))
        .query(ReadRetention {
            identity: DOCUMENT_IDENTITY.to_owned(),
        })
        .execute()
        .unwrap();
    assert_eq!(historical.rows()[0].retention_days, 18);
    assert!(matches!(
        completed.evidence(),
        WorthQuerySourceCompletion::Recovered {
            publication: None,
            ..
        }
    ));
    assert_eq!(host.contacts.counts(), (2, 2));
    completed
        .close(&fresh)
        .unwrap_or_else(|(_, cause)| panic!("replay close: {cause:?}"));
}

#[path = "native_postures/retained_capacity.rs"]
mod native_postures;
