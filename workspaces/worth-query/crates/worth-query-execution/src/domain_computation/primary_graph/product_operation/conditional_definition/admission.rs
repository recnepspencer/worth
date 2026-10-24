//! One definition successor issuer for selected entries and admitted handlers.
use super::*;
use crate::basis::WorthQueryProductBranchLease;
use crate::domain_computation::primary_graph::{
    conditional_operation::QueryTemporalPredicateProvider, WorthQueryPrimaryGraphApplicationRuntime,
};

impl<Schema: ApplicationSchema> WorthQueryPrimaryGraphApplicationRuntime<Schema> {
    pub(in crate::domain_computation::primary_graph) fn admit_conditional_definition_on_product<
        Node,
        Clock,
        Provider,
    >(
        &self,
        product: &WorthQueryProductBranchLease,
        handle: &WorthQueryConditionalClockHandle<Schema, Node, Clock>,
        provider: Arc<Provider>,
    ) -> Result<
        WorthQueryAdmittedApplicationConditionalDefinition,
        WorthQueryApplicationConditionalDefinitionAdmissionDenial,
    >
    where
        Node: 'static,
        Provider: WorthQueryHostConditionalPredicateProvider<Node>,
    {
        let (anchor, request) = definition_request(self, product, handle, provider)
            .map_err(|cause| match cause {
                WorthQueryConditionalDefinitionPublicationDenial::ForeignConditionalOperation => WorthQueryApplicationConditionalDefinitionAdmissionDenial::ForeignConditionalOperation,
                WorthQueryConditionalDefinitionPublicationDenial::BridgePreparation(cause) => WorthQueryApplicationConditionalDefinitionAdmissionDenial::BridgePreparation(cause),
                _ => unreachable!("definition preparation does not publish"),
            })?;
        Ok(WorthQueryAdmittedApplicationConditionalDefinition::new(
            self,
            product,
            *handle.binding_identity_digest(),
            anchor,
            request,
        ))
    }
}

pub(super) fn definition_request<Schema: ApplicationSchema, Node: 'static, Clock, Provider>(
    application: &WorthQueryPrimaryGraphApplicationRuntime<Schema>,
    product: &WorthQueryProductBranchLease,
    handle: &WorthQueryConditionalClockHandle<Schema, Node, Clock>,
    provider: Arc<Provider>,
) -> Result<
    (
        Arc<worth_runtime_bridge::facade::BridgeInstalledConditionalLowering>,
        worth_runtime_bridge::facade::BridgeOwnedConditionalInstallationRequest,
    ),
    WorthQueryConditionalDefinitionPublicationDenial,
>
where
    Provider: WorthQueryHostConditionalPredicateProvider<Node>,
{
    let operation = application
        .conditional_operations
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner)
        .admit_clock(handle.binding_identity(), handle.lease())
        .ok_or(WorthQueryConditionalDefinitionPublicationDenial::ForeignConditionalOperation)?;
    let operation_anchor = operation.operation_anchor();
    let bridge_root = application.bridge.conditional_operations();
    let bridge = bridge_root
        .read()
        .unwrap_or_else(std::sync::PoisonError::into_inner);
    let exact = bridge
        .admit_exact_conditional_signal_basis(&operation_anchor, product.signal_basis())
        .map_err(WorthQueryConditionalDefinitionPublicationDenial::BridgePreparation)?;
    let predecessor = exact.installed_lowering();
    let request = predecessor.successor_with_wake_provider(QueryTemporalPredicateProvider::<
        Node,
        Provider,
    >::new(
        provider,
        Arc::clone(handle.node_authority()),
    ));
    Ok((predecessor, request))
}

impl std::fmt::Display for WorthQueryApplicationConditionalDefinitionAdmissionDenial {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            formatter,
            "conditional definition admission refused: {self:?}"
        )
    }
}
impl std::error::Error for WorthQueryApplicationConditionalDefinitionAdmissionDenial {}
