//! Retained no-output delivery reads the native original World occurrence.
pub use super::{fixture::*, host, readback::*, schema::*};
pub use crate::facade::certification::document_retention_model::operator_identity::{
    authenticate_operator, request_scope,
};
pub use std::num::NonZeroUsize;
pub use worth_query_host::facade::{
    application_entry::*, declaration::application_program::*, declaration::application_schema::*,
};
pub struct RetainedProgram;
pub struct RetainedDelivery;
impl ApplicationActionDelivery<SourceOnlyRetentionSchema, SetRetentionBinding>
    for RetainedDelivery
{
    type Mode = ApplicationRetainedCommit;
    const IDENTITY: &'static str = "source-only-retention.retained-delivery.v1";
}
pub type Action = ApplicationActionReference<
    SourceOnlyRetentionSchema,
    ApplicationRootComposition,
    DocumentRetentionFeature,
    SetRetentionBinding,
    RetainedDelivery,
>;
pub type Operation = WorthQuerySourceOnlyOperation<
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
pub fn operation(
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
pub fn observation(
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

#[path = "native_postures/retained_capacity.rs"]
pub mod native_postures;
