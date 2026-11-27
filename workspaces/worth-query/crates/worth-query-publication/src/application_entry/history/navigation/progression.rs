//! Every progression uses fresh current admission; the owned handle survives refusal.
use super::*;

impl<S, H, N, P, Outputs> WorthQueryHistoryNavigation<'_, '_, S, H, N, P, Outputs>
where
    S: ApplicationSchema,
    H: ApplicationHistoryBinding<S>,
    N: ApplicationHistoryNavigationBinding<S, H>,
    P: WorthQueryHistoryPayloadMode<S, H>,
    Outputs: WorthQueryHistoryOutputsMode<S, H>,
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
        self.advance_in(request, Ungoverned)
    }

    /// The governed navigation entry, kept for Part B's `.expected()` path
    /// (tree-based undo 2E, ruling 11), which supplies its capability here.
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
        self.advance_in(request, Governed { capability, operation, input })
    }

    fn advance_in(
        self,
        request: &WorthQueryApplicationRequest<'_, '_, '_, S>,
        authority: impl MetadataAuthority<S, H>,
    ) -> WorthQueryHistoryNavigationAdvance<Self> {
        match read::with_read(self.document, request, authority, |phase, _, read| {
            self.document.outputs.advance::<N>(
                phase,
                request,
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
        let context = match DocumentRequestContext::bind(&self.document.owner, request) {
            Ok(context) => context,
            Err(cause) => return Err((self, Denial::History(cause))),
        };
        match context.advance(|phase| Ok(self.document.outputs.discard(phase, &self.navigation))) {
            Ok(Ok(())) => Ok(()),
            Ok(Err(cause)) => Err((self, cause)),
            Err(cause) => Err((self, Denial::History(cause))),
        }
    }
}
