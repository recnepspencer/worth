//! A document that keeps its installed runtime alive is stored beside that runtime.
use super::*;
use worth_query_declaration::facade::application_program::ApplicationProgramDefinition;
use worth_query_execution::facade::application_installation::WorthQueryProgramApplicationRuntime;
use worth_query_execution::facade::product::WorthQueryOwnedProductHead;

/// An attached document that holds its installed runtime alive instead of borrowing it.
///
/// It is the ordinary document with no runtime borrow, so every document
/// operation applies unchanged and an application can keep it in the same
/// owner as its runtime.
/// ```
/// use worth_query_declaration::facade::{
///     application_history::ApplicationHistoryBinding, application_schema::ApplicationSchema,
/// };
/// use worth_query_execution::facade::application_installation::WorthQueryProgramApplicationRuntime;
/// use worth_query_publication::facade::application_entry::WorthQueryOwnedApplicationHistoryDocument;
/// struct Session<S: ApplicationSchema, P, H: ApplicationHistoryBinding<S>> {
///     runtime: WorthQueryProgramApplicationRuntime<S, P>,
///     document: Option<WorthQueryOwnedApplicationHistoryDocument<S, H>>,
/// }
/// fn open<S: ApplicationSchema, P, H: ApplicationHistoryBinding<S>>(
///     runtime: WorthQueryProgramApplicationRuntime<S, P>,
///     document: WorthQueryOwnedApplicationHistoryDocument<S, H>,
/// ) -> Session<S, P, H> {
///     Session { runtime, document: Some(document) }
/// }
/// ```
/// While the document is open the runtime's `close_conditional_runtime`
/// refuses `RuntimeKeepaliveRetained`. Closing the document releases the hold.
///
/// It stays move-only.
/// ```compile_fail,E0382
/// use worth_query_declaration::facade::{
///     application_history::ApplicationHistoryBinding, application_schema::ApplicationSchema,
/// };
/// use worth_query_publication::facade::application_entry::WorthQueryOwnedApplicationHistoryDocument;
/// fn duplicate<S: ApplicationSchema, H: ApplicationHistoryBinding<S>>(
///     document: WorthQueryOwnedApplicationHistoryDocument<S, H>,
/// ) {
///     let owner = document;
///     drop(document);
///     drop(owner);
/// }
/// ```
/// Only attachment through the installed runtime issues one.
/// ```compile_fail,E0451
/// use worth_query_declaration::facade::{
///     application_history::ApplicationHistoryBinding, application_schema::ApplicationSchema,
/// };
/// use worth_query_publication::facade::application_entry::WorthQueryOwnedApplicationHistoryDocument;
/// fn counterfeit<S: ApplicationSchema, H: ApplicationHistoryBinding<S>>(
/// ) -> WorthQueryOwnedApplicationHistoryDocument<S, H> {
///     WorthQueryOwnedApplicationHistoryDocument::<S, H> {
///         owner: todo!(),
///         access: todo!(),
///         payload: todo!(),
///         outputs: todo!(),
///     }
/// }
/// ```
pub type WorthQueryOwnedApplicationHistoryDocument<
    S,
    H,
    Payload = OrdinaryHistoryPayload,
    Outputs = WorthQueryHistoryNoOutputs,
> = WorthQueryApplicationHistoryDocument<'static, S, H, Payload, Outputs>;

impl<S: ApplicationSchema> WorthQueryApplicationRequest<'_, '_, '_, S> {
    /// Attach the owned slot as a document that keeps `program` alive.
    ///
    /// The request and the head must both belong to `program`. A refusal
    /// returns the original head, still borrowing its runtime, and leaves no
    /// hold on the runtime.
    pub fn attach_owned_history<'head, H, P>(
        &self,
        program: &WorthQueryProgramApplicationRuntime<S, P>,
        phase: &WorthQueryAdvancementPhase<'_>,
        head: WorthQueryOwnedProductHead<'head, S>,
        access: H::Access,
    ) -> Result<
        WorthQueryOwnedApplicationHistoryDocument<S, H>,
        WorthQueryApplicationHistoryAttachmentFailure<'head, S>,
    >
    where
        H: ApplicationHistoryBinding<S>,
        P: ApplicationProgramDefinition<S>,
        Scope<S, H>: ApplicationQueryScopeResolution<S, Principal<S, H>>,
        Rows<S, H>: WorthQueryApplicationProjection<S, Query<S, H>>,
    {
        if !std::ptr::eq(self.application, program.runtime()) || !head.belongs_to(program.runtime())
        {
            return Err(WorthQueryApplicationHistoryAttachmentFailure {
                head,
                denial: WorthQueryApplicationHistoryDenial::ForeignRuntime,
            });
        }
        self.attach_history_payload(
            phase,
            head,
            access,
            OrdinaryHistoryPayload,
            |_, head, read| {
                program.attach_owned_document_history(
                    phase,
                    &program_publication_access(),
                    head,
                    read,
                )
            },
        )
    }
}
