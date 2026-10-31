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
        let result = self.query(access.clone()).with_admitted_query(
            phase,
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
            Ok(owner) => Ok(WorthQueryApplicationHistoryDocument { owner, access }),
            Err(denial) => Err(WorthQueryApplicationHistoryAttachmentFailure {
                head: head.expect("refused attachment retains its original head"),
                denial,
            }),
        }
    }
}
