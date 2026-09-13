use super::{runtime_installation::TestRuntimeInstaller, WorthQueryInMemoryTestRuntimeBuilder};

impl WorthQueryInMemoryTestRuntimeBuilder {
    pub fn owned_bridge_async_declaration(
        mut self,
        declaration: crate::runtime::WorthQueryOwnedAsyncRequestDeclaration,
    ) -> Self {
        self.runtime_installers
            .push(TestRuntimeInstaller::Immediate(Box::new(move |builder| {
                builder.owned_bridge_async_declaration(declaration)
            })));
        self
    }

    pub fn owned_conditional_runtime(
        mut self,
        bridge: worth_runtime_bridge::facade::RuntimeBridge,
        resources: crate::runtime::WorthQueryConditionalExecutionResources,
    ) -> Self {
        self.runtime_installers
            .push(TestRuntimeInstaller::Immediate(Box::new(move |builder| {
                builder.owned_conditional_runtime_for_test(bridge, resources)
            })));
        self
    }

    pub fn owned_topology_conditional_node<D, O, F, G, P>(
        mut self,
        domain: D,
        operation: O,
        family: F,
        graph: G,
        location: crate::domain_installation::WorthQueryConditionalNodeLocation,
        dependencies: Vec<
            crate::domain_installation::WorthQueryOwnedConditionalDependencyInstallation,
        >,
        providers: worth_runtime_bridge::facade::BridgeConditionalProviderSet,
        compute: P,
    ) -> Self
    where
        D: 'static,
        O: 'static,
        F: 'static,
        G: 'static,
        P: crate::domain_installation::WorthQueryConditionalNodeComputeProvider<D, O, F>,
    {
        self.runtime_installers
            .push(TestRuntimeInstaller::Immediate(Box::new(move |builder| {
                builder.owned_topology_conditional_node(
                    domain,
                    operation,
                    family,
                    graph,
                    location,
                    dependencies,
                    providers,
                    compute,
                )
            })));
        self
    }

    #[allow(clippy::too_many_arguments)]
    pub fn conditional_node<D, O, F, G, P>(
        mut self,
        domain: D,
        operation: O,
        family: F,
        graph: G,
        location: crate::domain_installation::WorthQueryConditionalNodeLocation,
        dependencies: Vec<crate::domain_installation::WorthQueryConditionalDependencyInstallation>,
        providers: worth_runtime_bridge::facade::BridgeConditionalProviderSet,
        compute: P,
    ) -> Self
    where
        D: 'static,
        O: 'static,
        F: 'static,
        G: 'static,
        P: crate::domain_installation::WorthQueryConditionalNodeComputeProvider<D, O, F>,
    {
        self.runtime_installers
            .push(TestRuntimeInstaller::Immediate(Box::new(move |builder| {
                builder.conditional_node(
                    domain,
                    operation,
                    family,
                    graph,
                    location,
                    dependencies,
                    providers,
                    compute,
                )
            })));
        self
    }
}
