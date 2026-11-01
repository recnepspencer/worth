//! Target, expected selection and key precede the admission that issues a live attempt.
use super::*;

pub struct WorthQueryHistoryNavigationBuilder<
    'd,
    'r,
    S: ApplicationSchema,
    H: ApplicationHistoryBinding<S>,
    N: ApplicationHistoryNavigationBinding<S, H>,
    P: WorthQueryHistoryPayloadMode<S, H> = OrdinaryHistoryPayload,
    Outputs = WorthQueryHistoryNoOutputs,
> {
    document: &'d WorthQueryApplicationHistoryDocument<'r, S, H, P, Outputs>,
    marker: std::marker::PhantomData<fn() -> N>,
}
pub struct WorthQueryHistoryCheckoutTarget<
    'd,
    'r,
    S: ApplicationSchema,
    H: ApplicationHistoryBinding<S>,
    N: ApplicationHistoryNavigationBinding<S, H>,
    P: WorthQueryHistoryPayloadMode<S, H> = OrdinaryHistoryPayload,
    Outputs = WorthQueryHistoryNoOutputs,
> {
    builder: WorthQueryHistoryNavigationBuilder<'d, 'r, S, H, N, P, Outputs>,
    node: WorthQueryHistoryNodeId,
}
pub struct WorthQueryHistoryExpectedCheckout<
    'd,
    'r,
    S: ApplicationSchema,
    H: ApplicationHistoryBinding<S>,
    N: ApplicationHistoryNavigationBinding<S, H>,
    P: WorthQueryHistoryPayloadMode<S, H> = OrdinaryHistoryPayload,
    Outputs = WorthQueryHistoryNoOutputs,
> {
    target: WorthQueryHistoryCheckoutTarget<'d, 'r, S, H, N, P, Outputs>,
    expected: WorthQueryHistorySelection,
}
pub struct WorthQueryHistoryCheckoutDescription<
    'd,
    'r,
    S: ApplicationSchema,
    H: ApplicationHistoryBinding<S>,
    N: ApplicationHistoryNavigationBinding<S, H>,
    P: WorthQueryHistoryPayloadMode<S, H> = OrdinaryHistoryPayload,
    Outputs = WorthQueryHistoryNoOutputs,
> {
    checkout: WorthQueryHistoryExpectedCheckout<'d, 'r, S, H, N, P, Outputs>,
    key: u128,
}

impl<
        'r,
        S: ApplicationSchema,
        H: ApplicationHistoryBinding<S>,
        P: WorthQueryHistoryPayloadMode<S, H>,
        Outputs,
    > WorthQueryApplicationHistoryDocument<'r, S, H, P, Outputs>
{
    pub fn navigate<N: ApplicationHistoryNavigationBinding<S, H>>(
        &self,
    ) -> WorthQueryHistoryNavigationBuilder<'_, 'r, S, H, N, P, Outputs> {
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
        Outputs,
    > WorthQueryHistoryNavigationBuilder<'d, 'r, S, H, N, P, Outputs>
{
    pub fn checkout(
        self,
        node: WorthQueryHistoryNodeId,
    ) -> WorthQueryHistoryCheckoutTarget<'d, 'r, S, H, N, P, Outputs> {
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
        Outputs,
    > WorthQueryHistoryCheckoutTarget<'d, 'r, S, H, N, P, Outputs>
{
    pub fn expected_selection(
        self,
        expected: WorthQueryHistorySelection,
    ) -> WorthQueryHistoryExpectedCheckout<'d, 'r, S, H, N, P, Outputs> {
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
        Outputs,
    > WorthQueryHistoryExpectedCheckout<'d, 'r, S, H, N, P, Outputs>
{
    pub fn idempotency(
        self,
        key: u128,
    ) -> WorthQueryHistoryCheckoutDescription<'d, 'r, S, H, N, P, Outputs> {
        WorthQueryHistoryCheckoutDescription {
            checkout: self,
            key,
        }
    }
}

impl<'d, 'r, S, H, N, P, Outputs> WorthQueryHistoryCheckoutDescription<'d, 'r, S, H, N, P, Outputs>
where
    S: ApplicationSchema,
    H: ApplicationHistoryBinding<S>,
    N: ApplicationHistoryNavigationBinding<S, H>,
    P: WorthQueryHistoryPayloadMode<S, H>,
    Scope<S, H>: ApplicationQueryScopeResolution<S, Principal<S, H>>,
    Rows<S, H>: WorthQueryApplicationProjection<S, Query<S, H>>,
    Outputs: WorthQueryHistoryOutputsMode<S, H>,
{
    pub fn prepare(
        self,
        request: &WorthQueryApplicationRequest<'_, '_, '_, S>,
    ) -> Result<
        WorthQueryHistoryNavigation<'d, 'r, S, H, N, P, Outputs>,
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
        WorthQueryHistoryNavigation<'d, 'r, S, H, N, P, Outputs>,
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
        WorthQueryHistoryNavigation<'d, 'r, S, H, N, P, Outputs>,
        WorthQueryHistoryNavigationAdvanceDenial,
    > {
        let document = self.checkout.target.builder.document;
        let navigation = read::with_read(document, request, prepare, |phase, _, read| {
            document.outputs.prepare::<N>(
                phase,
                request,
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
