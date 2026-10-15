//! Original mutation meaning retained without request authority.
use crate::application_entry::mutation::request::WorthQueryMutationExpectedSource;
use worth_query_declaration::facade::application_operation::{
    ApplicationMutationBinding, ApplicationMutationIntent, ApplicationMutationScopeBinding,
    ApplicationMutationSourceExpectation,
};
use worth_query_declaration::facade::application_schema::TypedMutationPreconditions;
use worth_query_execution::facade::product::WorthQueryProductBranch;
use worth_query_installation::facade::ApplicationSchema;

pub(super) type Conditions<Schema, Intent> = TypedMutationPreconditions<
    Schema,
    <<Intent as ApplicationMutationIntent<Schema>>::Binding as ApplicationMutationBinding<
        Schema,
    >>::Operation,
    <<<Intent as ApplicationMutationIntent<Schema>>::Binding as ApplicationMutationBinding<
        Schema,
    >>::ScopeBinding as ApplicationMutationScopeBinding<Schema>>::Scope,
>;
pub(super) type SourceQuery<Schema, Intent> =
    <<<Intent as ApplicationMutationIntent<Schema>>::Binding as ApplicationMutationBinding<
        Schema,
    >>::SourceExpectation as ApplicationMutationSourceExpectation<Schema>>::Query;

/// The complete original description. It carries meaning and custody, never a principal or permission.
///
/// Original intent replacement is unavailable after retention.
/// Retained preconditions cannot be omitted or replaced on retry.
/// ```compile_fail,E0616
/// use worth_query_publication::facade::application_entry::WorthQueryOperationDescription;
/// use worth_query_declaration::facade::{application_schema::ApplicationSchema, application_operation::ApplicationMutationIntent};
/// fn replace_conditions<S: ApplicationSchema, I: ApplicationMutationIntent<S>>(mut description: WorthQueryOperationDescription<S, I>) {
///     description.preconditions = description.preconditions;
/// }
/// ```
/// A retained result set cannot be replaced with a row or another selection.
/// ```compile_fail,E0616
/// use worth_query_publication::facade::application_entry::WorthQueryOperationDescription;
/// use worth_query_declaration::facade::{application_schema::ApplicationSchema, application_operation::ApplicationMutationIntent};
/// fn replace_source<S: ApplicationSchema, I: ApplicationMutationIntent<S>>(mut description: WorthQueryOperationDescription<S, I>) {
///     description.source = None;
/// }
/// ```
/// Fresh authority cannot change the original operation key.
/// ```compile_fail,E0616
/// use worth_query_publication::facade::application_entry::WorthQueryOperationDescription;
/// use worth_query_declaration::facade::{application_schema::ApplicationSchema, application_operation::ApplicationMutationIntent};
/// fn replace_key<S: ApplicationSchema, I: ApplicationMutationIntent<S>>(mut description: WorthQueryOperationDescription<S, I>) {
///     description.key = description.key;
/// }
/// ```
/// The original input is likewise private.
/// ```compile_fail,E0616
/// use worth_query_publication::facade::application_entry::WorthQueryOperationDescription;
/// use worth_query_declaration::facade::{application_schema::ApplicationSchema, application_operation::ApplicationMutationIntent};
/// fn replace_original<S: ApplicationSchema, I: ApplicationMutationIntent<S>>(mut description: WorthQueryOperationDescription<S, I>, replacement: I) {
///     description.intent = replacement;
/// }
/// ```
/// Descriptions cannot be cloned to create a competing original operation.
/// ```compile_fail,E0599
/// use worth_query_publication::facade::application_entry::WorthQueryOperationDescription;
/// use worth_query_declaration::facade::{application_schema::ApplicationSchema, application_operation::ApplicationMutationIntent};
/// fn duplicate<S: ApplicationSchema, I: ApplicationMutationIntent<S>>(description: WorthQueryOperationDescription<S, I>) {
///     let _other = description.clone();
/// }
/// ```
/// Read delivery needs actual native settlement.
/// ```compile_fail,E0599
/// use worth_query_publication::facade::application_entry::{WorthQueryOperationDescription, WorthQueryApplicationRequest};
/// use worth_query_declaration::facade::{application_schema::ApplicationSchema, application_operation::ApplicationMutationIntent};
/// fn premature_read<S: ApplicationSchema, I: ApplicationMutationIntent<S>>(description: WorthQueryOperationDescription<S, I>, request: &WorthQueryApplicationRequest<'_, '_, '_, S>) {
///     let _ = description.read_current(request);
/// }
/// ```
/// Source preparation cannot be skipped when constructing an original description.
/// ```compile_fail,E0599
/// use worth_query_publication::facade::application_entry::WorthQueryOperationBuilder;
/// use worth_query_declaration::facade::{application_schema::ApplicationSchema, application_operation::ApplicationMutationIntent};
/// fn skip_source<S: ApplicationSchema, I: ApplicationMutationIntent<S>>(request: WorthQueryOperationBuilder<'_, '_, '_, S, I>) {
///     let _ = request.idempotency_owned(todo!());
/// }
/// ```
#[must_use = "retain the original descriptor as a declared operation before advancing"]
pub struct WorthQueryOperationDescription<
    Schema: ApplicationSchema,
    Intent: ApplicationMutationIntent<Schema>,
> {
    pub(super) intent: Intent,
    // Freeze input custody once; attempts share it without copying domain input or authority.
    pub(super) encoded_input: Result<
        worth_query_declaration::facade::application_operation::ApplicationEncodedInput<
            <Intent::Binding as ApplicationMutationBinding<Schema>>::InputBinding,
        >,
        worth_query_declaration::facade::application_schema::ApplicationValueEncodeDenial,
    >,
    pub(super) key: <Intent::Binding as ApplicationMutationBinding<Schema>>::IdempotencyKey,
    pub(super) preconditions: Conditions<Schema, Intent>,
    pub(super) source: Option<WorthQueryMutationExpectedSource<SourceQuery<Schema, Intent>>>,
    pub(super) branch: WorthQueryProductBranch,
}
