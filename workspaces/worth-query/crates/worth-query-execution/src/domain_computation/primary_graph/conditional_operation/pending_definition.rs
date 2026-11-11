//! Install authored declarations only; no clock, wake, temporal intent or operation is created.
use super::definition_declarations::{
    ConditionalDefinitionDeclarations, InstalledDefinitionDeclaration,
};
use super::installation::{
    route_binding::InstalledConditionalRoute, ConditionalClockLease,
    WorthQueryConditionalRuntimeInstallationDenial as D,
    WorthQueryConditionalRuntimeInstallationDenialKind as Kind,
};
use std::sync::Arc;
use worth_query_installation::facade::{
    ApplicationSchema, WorthQueryHostConditionalPredicateProvider,
    WorthQueryInstalledGraphParticipationAuthority, WorthQueryInstalledHostConditionalProvider,
};
use worth_runtime_bridge::facade::BridgeConditionalRuntimeBuilder;

pub(in crate::domain_computation::primary_graph) trait PendingDefinitionDeclaration {
    fn install(
        self: Box<Self>,
        builder: &mut BridgeConditionalRuntimeBuilder,
        graph: &WorthQueryInstalledGraphParticipationAuthority,
    ) -> Result<InstalledDefinitionDeclaration, D>;
}
pub(super) struct PendingTypedDefinition<S, A, Input, Def, Op, Fact, Node, P> {
    pub(super) provider:
        WorthQueryInstalledHostConditionalProvider<S, A, Input, Def, Op, Fact, Node, P>,
    pub(super) identity: Arc<str>,
    pub(super) lease: Arc<ConditionalClockLease>,
}
impl<
        S: ApplicationSchema + 'static,
        A: 'static,
        Input: 'static,
        Def: 'static,
        Op: 'static,
        Fact: 'static,
        Node: 'static,
        P,
    > PendingDefinitionDeclaration for PendingTypedDefinition<S, A, Input, Def, Op, Fact, Node, P>
where
    P: WorthQueryHostConditionalPredicateProvider<Node>,
{
    fn install(
        self: Box<Self>,
        builder: &mut BridgeConditionalRuntimeBuilder,
        graph: &WorthQueryInstalledGraphParticipationAuthority,
    ) -> Result<InstalledDefinitionDeclaration, D> {
        let request =
            super::predicate_admission::prepare_host_predicate_installation(&self.provider, graph)?;
        request
            .providers
            .authored_descriptions()
            .map_err(|cause| D::new(Kind::AuthoredProviderSourceRequired, format!("{cause:?}")))?;
        let lowering = builder
            .install_owned_conditional(request)
            .map_err(|cause| {
                D::new(
                    Kind::BridgeConditional(Box::new(cause.kind())),
                    cause.detail(),
                )
            })?;
        Ok(InstalledDefinitionDeclaration {
            identity: self.identity,
            lease: self.lease,
            route: InstalledConditionalRoute::from_installation(lowering),
        })
    }
}
pub(in crate::domain_computation::primary_graph) fn install_pending_definitions(
    definitions: Vec<Box<dyn PendingDefinitionDeclaration>>,
    builder: &mut BridgeConditionalRuntimeBuilder,
    graph: &WorthQueryInstalledGraphParticipationAuthority,
) -> Result<ConditionalDefinitionDeclarations, D> {
    let mut result = ConditionalDefinitionDeclarations::default();
    for pending in definitions {
        result.install(pending.install(builder, graph)?);
    }
    Ok(result)
}
