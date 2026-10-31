//! Target, expected selection and key precede the admission that issues a live attempt.
use super::*;

pub struct WorthQueryHistoryNavigationBuilder<
    'd,
    'r,
    S: ApplicationSchema,
    H: ApplicationHistoryBinding<S>,
    N: ApplicationHistoryNavigationBinding<S, H>,
    P: WorthQueryHistoryPayloadMode<S, H> = OrdinaryHistoryPayload,
> {
    document: &'d WorthQueryApplicationHistoryDocument<'r, S, H, P>,
    marker: std::marker::PhantomData<fn() -> N>,
}
pub struct WorthQueryHistoryCheckoutTarget<
    'd,
    'r,
    S: ApplicationSchema,
    H: ApplicationHistoryBinding<S>,
    N: ApplicationHistoryNavigationBinding<S, H>,
    P: WorthQueryHistoryPayloadMode<S, H> = OrdinaryHistoryPayload,
> {
    builder: WorthQueryHistoryNavigationBuilder<'d, 'r, S, H, N, P>,
    node: WorthQueryHistoryNodeId,
}
pub struct WorthQueryHistoryExpectedCheckout<
    'd,
    'r,
    S: ApplicationSchema,
    H: ApplicationHistoryBinding<S>,
    N: ApplicationHistoryNavigationBinding<S, H>,
    P: WorthQueryHistoryPayloadMode<S, H> = OrdinaryHistoryPayload,
> {
    target: WorthQueryHistoryCheckoutTarget<'d, 'r, S, H, N, P>,
    expected: WorthQueryHistorySelection,
}
pub struct WorthQueryHistoryCheckoutDescription<
    'd,
    'r,
    S: ApplicationSchema,
    H: ApplicationHistoryBinding<S>,
    N: ApplicationHistoryNavigationBinding<S, H>,
    P: WorthQueryHistoryPayloadMode<S, H> = OrdinaryHistoryPayload,
> {
    checkout: WorthQueryHistoryExpectedCheckout<'d, 'r, S, H, N, P>,
    key: u128,
}

impl<
        'r,
        S: ApplicationSchema,
        H: ApplicationHistoryBinding<S>,
        P: WorthQueryHistoryPayloadMode<S, H>,
    > WorthQueryApplicationHistoryDocument<'r, S, H, P>
{
    pub fn navigate<N: ApplicationHistoryNavigationBinding<S, H>>(
        &self,
    ) -> WorthQueryHistoryNavigationBuilder<'_, 'r, S, H, N, P> {
        WorthQueryHistoryNavigationBuilder {
            document: self,
            marker: std::marker::PhantomData,
        }
    }
}
impl<
        'd,
        'r,
        S: ApplicationSchema,
        H: ApplicationHistoryBinding<S>,
        N: ApplicationHistoryNavigationBinding<S, H>,
        P: WorthQueryHistoryPayloadMode<S, H>,
    > WorthQueryHistoryNavigationBuilder<'d, 'r, S, H, N, P>
{
    pub fn checkout(
        self,
        node: WorthQueryHistoryNodeId,
    ) -> WorthQueryHistoryCheckoutTarget<'d, 'r, S, H, N, P> {
        WorthQueryHistoryCheckoutTarget {
            builder: self,
            node,
        }
    }
}
impl<
        'd,
        'r,
        S: ApplicationSchema,
        H: ApplicationHistoryBinding<S>,
        N: ApplicationHistoryNavigationBinding<S, H>,
        P: WorthQueryHistoryPayloadMode<S, H>,
    > WorthQueryHistoryCheckoutTarget<'d, 'r, S, H, N, P>
{
    pub fn expected_selection(
        self,
        expected: WorthQueryHistorySelection,
    ) -> WorthQueryHistoryExpectedCheckout<'d, 'r, S, H, N, P> {
        WorthQueryHistoryExpectedCheckout {
            target: self,
            expected,
        }
    }
}
impl<
        'd,
        'r,
        S: ApplicationSchema,
        H: ApplicationHistoryBinding<S>,
        N: ApplicationHistoryNavigationBinding<S, H>,
        P: WorthQueryHistoryPayloadMode<S, H>,
    > WorthQueryHistoryExpectedCheckout<'d, 'r, S, H, N, P>
{
    pub fn idempotency(
        self,
        key: u128,
    ) -> WorthQueryHistoryCheckoutDescription<'d, 'r, S, H, N, P> {
        WorthQueryHistoryCheckoutDescription {
            checkout: self,
            key,
        }
    }
}

impl<'d, 'r, S, H, N, P> WorthQueryHistoryCheckoutDescription<'d, 'r, S, H, N, P>
where
    S: ApplicationSchema,
    H: ApplicationHistoryBinding<S>,
    N: ApplicationHistoryNavigationBinding<S, H>,
    P: WorthQueryHistoryPayloadMode<S, H>,
    Scope<S, H>: ApplicationQueryScopeResolution<S, Principal<S, H>>,
    Rows<S, H>: WorthQueryApplicationProjection<S, Query<S, H>>,
{
    pub fn prepare(
        self,
        request: &WorthQueryApplicationRequest<'_, '_, '_, S>,
    ) -> Result<
        WorthQueryHistoryNavigation<'d, 'r, S, H, N, P>,
        WorthQueryHistoryNavigationAdvanceDenial,
    > {
        self.prepare_with(request, |_, _, read| Ok(read))
    }
    pub fn prepare_governed<C, O, I>(
        self,
        request: &WorthQueryApplicationRequest<'_, '_, '_, S>,
        capability: ApplicationCapabilityRef<S, C>,
        operation: ApplicationOperationRef<S, O, I>,
        input: I,
    ) -> Result<
        WorthQueryHistoryNavigation<'d, 'r, S, H, N, P>,
        WorthQueryHistoryNavigationAdvanceDenial,
    >
    where
        O: ApplicationOperationMarkerIdentity<S> + 'static,
        O::InputBinding: ApplicationStructuredValueBinding<Value = I>,
        I: ApplicationCapabilityRequest<S, C, Scope = ApplicationHistoryModel<S, H>> + 'static,
    {
        self.prepare_with(request, |phase, runtime, read| {
            read.with_metadata_capability(runtime, phase, capability, operation, input)
        })
    }
    fn prepare_with(
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
    ) -> Result<
        WorthQueryHistoryNavigation<'d, 'r, S, H, N, P>,
        WorthQueryHistoryNavigationAdvanceDenial,
    > {
        let document = self.checkout.target.builder.document;
        let navigation = read::with_read(document, request, prepare, |phase, runtime, read| {
            runtime.prepare_document_history_navigation::<H, N>(
                phase,
                &program_publication_access(),
                &document.owner,
                &document.access,
                read,
                self.checkout.expected,
                self.checkout.target.node,
                self.key,
            )
        })?;
        Ok(WorthQueryHistoryNavigation {
            document,
            navigation,
            key: self.key,
            marker: std::marker::PhantomData,
        })
    }
}
