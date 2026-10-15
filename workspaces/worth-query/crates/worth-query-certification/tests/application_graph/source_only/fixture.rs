//! A separately authored no-output program preserves the canonical retention programs.
use super::{host, schema::*};
use worth_query_host::facade::{
    application_entry::*, declaration::application_operation::*,
    declaration::application_program::*, declaration::application_schema::*,
};

pub struct SourceOnlyRetentionProgram;
pub struct DocumentRetentionFeature;
impl ApplicationFeature<SourceOnlyRetentionSchema> for DocumentRetentionFeature {
    type Inputs = ApplicationFeatureInputLeaf;
    const IDENTITY: &'static str = "source-only-retention.feature.v1";
}
// The original domain intent is deliberately move-only.
pub struct OriginalRetention {
    pub(super) input: SetRetentionInput,
}
impl ApplicationMutationIntent<SourceOnlyRetentionSchema> for OriginalRetention {
    type Binding = SetRetentionBinding;
    fn input(&self) -> &SetRetentionInput {
        &self.input
    }
    fn scope_binding(
        &self,
    ) -> <SetRetentionBinding as ApplicationMutationBinding<SourceOnlyRetentionSchema>>::ScopeBinding
    {
        <SetRetentionBinding as ApplicationMutationBinding<SourceOnlyRetentionSchema>>::ScopeBinding::new(DocumentIdentityField::reference(), self.input.identity.clone())
    }
}
pub struct RetentionCompletion;
impl ApplicationActionDelivery<SourceOnlyRetentionSchema, SetRetentionBinding>
    for RetentionCompletion
{
    type Mode = ApplicationSourceCompletion;
    const IDENTITY: &'static str = "retention-source-only-completion.v1";
}
pub type RetentionAction = ApplicationActionReference<
    SourceOnlyRetentionSchema,
    ApplicationRootComposition,
    DocumentRetentionFeature,
    SetRetentionBinding,
    RetentionCompletion,
>;
pub type Operation = WorthQuerySourceOnlyOperation<
    SourceOnlyRetentionSchema,
    OriginalRetention,
    SourceOnlyRetentionProgram,
    RetentionAction,
    ApplicationSourceCompletion,
>;
pub type Completed = WorthQueryCompletedSourceOperation<
    SourceOnlyRetentionSchema,
    OriginalRetention,
    SourceOnlyRetentionProgram,
    RetentionAction,
    ApplicationSourceCompletion,
>;

impl ApplicationProgramDefinition<SourceOnlyRetentionSchema> for SourceOnlyRetentionProgram {
    type Contributions = (SourceOnlyRetentionContribution,);
    type Outputs = ApplicationProgramOutputs<ApplicationNoOutputGraph>;
    type Rules = ApplicationRuleLeaf;
    const IDENTITY: ApplicationProgramIdentity =
        ApplicationProgramIdentity::new("worth.query.certification.retention-source-only.v1");
    fn feature_specs() -> Vec<ApplicationFeatureSpec> {
        vec![
            ApplicationFeatureSpec::root::<SourceOnlyRetentionSchema, DocumentRetentionFeature>()
                .operation_action::<SetRetentionBinding, RetentionCompletion>()
                .finish(),
        ]
    }
}
pub fn install() -> host::Host {
    host::install()
}
pub fn operation(
    host: &host::Host,
    request: &WorthQueryApplicationRequest<'_, '_, '_, SourceOnlyRetentionSchema>,
    days: u64,
    key: u64,
) -> Operation {
    operation_with_expected(host, request, days, key, DOCUMENT_IDENTITY)
}
pub fn operation_with_expected(
    host: &host::Host,
    request: &WorthQueryApplicationRequest<'_, '_, '_, SourceOnlyRetentionSchema>,
    days: u64,
    key: u64,
    expected: &str,
) -> Operation {
    let conditions = TypedMutationPreconditions::new().expect_fact(
        DocumentIdentityField::reference(),
        ApplicationEncodedScalarValue::try_new(expected.to_owned()).unwrap(),
    );
    request
        .mutate(OriginalRetention {
            input: SetRetentionInput {
                identity: DOCUMENT_IDENTITY.to_owned(),
                retention_days: days,
            },
        })
        .without_source()
        .preconditions(conditions)
        .idempotency_owned(key)
        .unwrap_or_else(|(_, _, denial)| panic!("input capture: {denial:?}"))
        .retain_operation::<SourceOnlyRetentionProgram, RetentionAction>(host)
        .unwrap_or_else(|(_, denial)| panic!("the exact no-output action constructs: {denial:?}"))
}
pub fn progress(
    operation: Operation,
    request: &WorthQueryApplicationRequest<'_, '_, '_, SourceOnlyRetentionSchema>,
) -> Operation {
    match operation.advance(request) {
        WorthQuerySourceOnlyAdvance::Progress(operation) => operation,
        WorthQuerySourceOnlyAdvance::Refused { denial, .. } => {
            panic!("expected native progress, got {denial:?}")
        }
        WorthQuerySourceOnlyAdvance::Terminal(terminal) => panic!(
            "expected native progress, got terminal {:?}",
            terminal.native_outcome()
        ),
        _ => panic!("expected retained native progress"),
    }
}
pub fn completed(
    operation: Operation,
    request: &WorthQueryApplicationRequest<'_, '_, '_, SourceOnlyRetentionSchema>,
) -> Completed {
    match operation.advance(request) {
        WorthQuerySourceOnlyAdvance::Completed(completed) => completed,
        WorthQuerySourceOnlyAdvance::Refused { denial, .. } => {
            panic!("expected completion, got {denial:?}")
        }
        WorthQuerySourceOnlyAdvance::Terminal(terminal) => panic!(
            "expected completion, got terminal {:?}",
            terminal.native_outcome()
        ),
        _ => panic!("expected actual source completion"),
    }
}
