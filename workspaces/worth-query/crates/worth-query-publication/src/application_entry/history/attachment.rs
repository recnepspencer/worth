use super::*;
use worth_query_execution::facade::product::WorthQueryOwnedProductHead;

impl<S: ApplicationSchema> WorthQueryApplicationRequest<'_, '_, '_, S> {
    /// Attach the actual owned slot using the declared READ on this current branch.
    /// A refusal returns the original head with the genuine admission/read denial.
    pub fn attach_history<'head, H: ApplicationHistoryBinding<S>>(
        &self,
        phase: &WorthQueryAdvancementPhase<'_>,
        head: WorthQueryOwnedProductHead<'head, S>,
        access: H::Access,
    ) -> Result<
        WorthQueryApplicationHistoryDocument<'head, S, H>,
        WorthQueryApplicationHistoryAttachmentFailure<'head, S>,
    >
    where
        Scope<S, H>: ApplicationQueryScopeResolution<S, Principal<S, H>>,
        Rows<S, H>: WorthQueryApplicationProjection<S, Query<S, H>>,
    {
        self.attach_history_payload(phase, head, access, OrdinaryHistoryPayload)
    }

    /// Attach with a reusable purpose for fresh governed payload READ admission.
    /// The document stores references and input, never a grant or approval.
    pub fn attach_history_with_payload<'head, H, C, O, I>(
        &self,
        phase: &WorthQueryAdvancementPhase<'_>,
        head: WorthQueryOwnedProductHead<'head, S>,
        access: H::Access,
        capability: worth_query_declaration::facade::application_capability::ApplicationCapabilityRef<S, C>,
        operation: worth_query_declaration::facade::application_schema::ApplicationOperationRef<S, O, I>,
        input: I,
    ) -> Result<
        WorthQueryApplicationHistoryDocument<'head, S, H, WorthQueryGovernedHistoryPayload<S, C, O, I>>,
        WorthQueryApplicationHistoryAttachmentFailure<'head, S>,
    >
    where
        H: ApplicationHistoryBinding<S>,
        O: worth_query_declaration::facade::application_schema::ApplicationOperationMarkerIdentity<S> + 'static,
        O::InputBinding: ApplicationStructuredValueBinding<Value = I>,
        I: worth_query_declaration::facade::application_capability::ApplicationCapabilityRequest<
            S, C, Scope = worth_query_declaration::facade::application_history::ApplicationHistoryModel<S, H>,
        > + Clone + 'static,
        Scope<S, H>: ApplicationQueryScopeResolution<S, Principal<S, H>>,
        Rows<S, H>: WorthQueryApplicationProjection<S, Query<S, H>>,
{
        self.attach_history_payload(
            phase,
            head,
            access,
            WorthQueryGovernedHistoryPayload {
                capability,
                operation,
                input,
            },
        )
    }

    fn attach_history_payload<'head, H, Payload>(
        &self,
        phase: &WorthQueryAdvancementPhase<'_>,
        head: WorthQueryOwnedProductHead<'head, S>,
        access: H::Access,
        payload: Payload,
    ) -> Result<
        WorthQueryApplicationHistoryDocument<'head, S, H, Payload>,
        WorthQueryApplicationHistoryAttachmentFailure<'head, S>,
    >
    where
        H: ApplicationHistoryBinding<S>,
        Payload: WorthQueryHistoryPayloadMode<S, H>,
        Scope<S, H>: ApplicationQueryScopeResolution<S, Principal<S, H>>,
        Rows<S, H>: WorthQueryApplicationProjection<S, Query<S, H>>,
    {
        if self.branch != head.branch() {
            return Err(WorthQueryApplicationHistoryAttachmentFailure {
                head,
                denial: WorthQueryApplicationHistoryDenial::RequestTargetMismatch,
            });
        }
        let installed = match self.application.installed_schema().installed_history::<H>() {
            Ok(installed) => installed,
            Err(cause) => {
                return Err(WorthQueryApplicationHistoryAttachmentFailure {
                    head,
                    denial: WorthQueryApplicationHistoryDenial::Installation(cause),
                })
            }
        };
        let mut head = Some(head);
        let result = payload.with_read(
            self.query(access.clone()),
            phase,
            None,
            |runtime, binding, external, plan| {
                let read = WorthQueryDocumentHistoryRead::new(&installed, binding, external, plan);
                Ok(runtime
                    .attach_document_history(
                        phase,
                        &program_publication_access(),
                        head.take().expect("one attachment transfer"),
                        read,
                    )
                    .map_err(|failure| {
                        let (original, denial) = failure.into_parts();
                        head = Some(original);
                        WorthQueryApplicationHistoryDenial::Document(denial)
                    }))
            },
        );
        let result = result
            .map_err(WorthQueryApplicationHistoryDenial::Query)
            .and_then(|result| result);
        match result {
            Ok(owner) => Ok(WorthQueryApplicationHistoryDocument {
                owner,
                access,
                payload,
            }),
            Err(denial) => Err(WorthQueryApplicationHistoryAttachmentFailure {
                head: head.expect("refused attachment retains its original head"),
                denial,
            }),
        }
    }
}
