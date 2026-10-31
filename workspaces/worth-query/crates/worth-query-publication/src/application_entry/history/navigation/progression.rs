//! Every progression uses fresh current admission; the owned handle survives refusal.
use super::*;

impl<S, H, N, P> WorthQueryHistoryNavigation<'_, '_, S, H, N, P>
where
    S: ApplicationSchema,
    H: ApplicationHistoryBinding<S>,
    N: ApplicationHistoryNavigationBinding<S, H>,
    P: WorthQueryHistoryPayloadMode<S, H>,
    Scope<S, H>: ApplicationQueryScopeResolution<S, Principal<S, H>>,
    Rows<S, H>: WorthQueryApplicationProjection<S, Query<S, H>>,
{
    pub fn posture(&self) -> WorthQueryHistoryNavigationPosture {
        self.navigation.posture()
    }

    pub fn advance(
        self,
        request: &WorthQueryApplicationRequest<'_, '_, '_, S>,
    ) -> WorthQueryHistoryNavigationAdvance<Self> {
        self.advance_with(request, |_, _, read| Ok(read))
    }

    pub fn advance_governed<C, O, I>(
        self,
        request: &WorthQueryApplicationRequest<'_, '_, '_, S>,
        capability: ApplicationCapabilityRef<S, C>,
        operation: ApplicationOperationRef<S, O, I>,
        input: I,
    ) -> WorthQueryHistoryNavigationAdvance<Self>
    where
        O: ApplicationOperationMarkerIdentity<S> + 'static,
        O::InputBinding: ApplicationStructuredValueBinding<Value = I>,
        I: ApplicationCapabilityRequest<S, C, Scope = ApplicationHistoryModel<S, H>> + 'static,
    {
        self.advance_with(request, |phase, runtime, read| {
            read.with_metadata_capability(runtime, phase, capability, operation, input)
        })
    }

    fn advance_with(
        self,
        request: &WorthQueryApplicationRequest<'_, '_, '_, S>,
        prepare: impl for<'read> FnOnce(
            &WorthQueryAdvancementPhase<'_>,
            &WorthQueryPrimaryGraphApplicationRuntime<S>,
            WorthQueryDocumentHistoryRead<'read, S, H>,
        ) -> Result<
            WorthQueryDocumentHistoryRead<'read, S, H>,
            WorthQueryDocumentHistoryDenial,
        >,
    ) -> WorthQueryHistoryNavigationAdvance<Self> {
        match read::with_read(self.document, request, prepare, |phase, runtime, read| {
            runtime.advance_document_history_navigation::<H, N>(
                phase,
                &program_publication_access(),
                &self.document.owner,
                &self.document.access,
                read,
                &self.navigation,
                self.key,
            )
        }) {
            Ok(Some(completed)) => WorthQueryHistoryNavigationAdvance::Selected(completed),
            Ok(None) => WorthQueryHistoryNavigationAdvance::Pending(self),
            Err(cause) => WorthQueryHistoryNavigationAdvance::Refused { cause, owner: self },
        }
    }

    /// Disposal owns no model read or new navigation effect. It can release this
    /// attempt after navigation permission has been revoked, using a fresh scope.
    pub fn discard(
        self,
        request: &WorthQueryApplicationRequest<'_, '_, '_, S>,
    ) -> Result<(), (Self, WorthQueryHistoryNavigationAdvanceDenial)> {
        use WorthQueryHistoryNavigationAdvanceDenial as Denial;
        let runtime = self.document.owner.application();
        if !std::ptr::eq(runtime, request.application) {
            return Err((
                self,
                Denial::History(WorthQueryApplicationHistoryDenial::ForeignRuntime),
            ));
        }
        let result = runtime.with_application_advancement(request.scope, |phase| {
            self.navigation
                .discard(&phase, &program_publication_access())
        });
        match result {
            Ok(Ok(())) => Ok(()),
            Ok(Err(cause)) => Err((self, Denial::Navigation(cause))),
            Err(cause) => Err((
                self,
                Denial::History(WorthQueryApplicationHistoryDenial::ExecutionRequest(cause)),
            )),
        }
    }
}
