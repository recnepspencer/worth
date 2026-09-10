use super::WorthQueryRuntimeBuilder;

impl WorthQueryRuntimeBuilder {
    pub(crate) fn installed_product_bridge(
        mut self,
        bridge: worth_runtime_bridge::facade::RuntimeBridge,
        resources: super::super::WorthQueryConditionalExecutionResources,
    ) -> Self {
        self.conditional_runtime_bridge = Some(bridge);
        self.conditional_execution_resources = Some(resources);
        self
    }

    pub fn conditional_execution_resources(
        mut self,
        resources: super::super::WorthQueryConditionalExecutionResources,
    ) -> Self {
        self.conditional_execution_resources = Some(resources);
        self
    }

    pub(crate) fn install_seeded_conditional_runtime(
        mut self,
        bridge: worth_runtime_bridge::facade::RuntimeBridge,
        graph: worth_signal::facade::SignalGraph,
        resources: super::super::WorthQueryConditionalExecutionResources,
    ) -> Self {
        self.conditional_runtime_bridge = Some(bridge);
        self.conditional_signal_graph = Some(Box::new(graph));
        self.conditional_execution_resources = Some(resources);
        self
    }

    /// Supplies the one Signal graph retained by the Query runtime's existing
    /// Runtime Bridge. Conditional declarations cannot build without it.
    pub fn conditional_signal_graph(mut self, graph: worth_signal::facade::SignalGraph) -> Self {
        self.conditional_signal_graph = Some(Box::new(graph));
        self
    }

    #[allow(clippy::too_many_arguments)]
    pub fn conditional_node<D, O, F, G, P>(
        mut self,
        _domain: D,
        _operation: O,
        _family: F,
        _graph: G,
        location: worth_query_installation::facade::WorthQueryConditionalNodeLocation,
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
        self.pending_conditional_installations.push(Box::new(
            crate::domain_installation::PendingConditionalNode::<D, O, F, G, P>::new(
                location,
                dependencies,
                providers,
                compute,
            ),
        ));
        self
    }

    pub(crate) fn owned_conditional_runtime_for_test(
        mut self,
        bridge: worth_runtime_bridge::facade::RuntimeBridge,
        resources: super::super::WorthQueryConditionalExecutionResources,
    ) -> Self {
        self.conditional_runtime_bridge = Some(bridge);
        self.conditional_execution_resources = Some(resources);
        self
    }

    pub fn owned_topology_conditional_node<D, O, F, G, P>(
        mut self,
        _domain: D,
        _operation: O,
        _family: F,
        _graph: G,
        location: worth_query_installation::facade::WorthQueryConditionalNodeLocation,
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
        self.pending_conditional_installations.push(Box::new(
            crate::domain_installation::PendingOwnedConditionalNode::<D, O, F, G, P>::new(
                location,
                dependencies,
                providers,
                compute,
            ),
        ));
        self
    }

    pub(super) fn install_conditional_execution(
        conditional_runtime_bridge: Option<worth_runtime_bridge::facade::RuntimeBridge>,
        conditional_signal_graph: Option<Box<worth_signal::facade::SignalGraph>>,
        resources: Option<super::super::WorthQueryConditionalExecutionResources>,
        pending_conditional_installations: &[Box<
            dyn crate::domain_installation::PendingConditionalInstallation,
        >],
        pending_owned_async_declarations: &[super::super::WorthQueryOwnedAsyncRequestDeclaration],
        domains: &crate::domain_installation::WorthQueryDomainInstallationRegistry,
        graphs: &crate::domain_installation::WorthQueryInstalledGraphParticipationRegistry,
    ) -> Result<
        (
            Option<worth_runtime_bridge::facade::BridgeSealedRuntimeAssembly>,
            crate::domain_installation::WorthQueryConditionalExecutionRegistry,
            Vec<(
                super::super::WorthQueryOwnedAsyncRequestDeclaration,
                worth_runtime_bridge::facade::LoweredBridgeAsyncSourceDeclaration,
            )>,
        ),
        crate::runtime::WorthQueryRuntimeError,
    > {
        let expected = domains
            .execution_index()
            .domain_operation_execution_descriptors()
            .iter()
            .map(|descriptor| descriptor.conditional_node_count)
            .sum::<usize>();
        if expected != pending_conditional_installations.len() {
            return Err(conditional_installation_error(format!(
                "installed declarations require {expected} conditional registrations, found {}",
                pending_conditional_installations.len()
            )));
        }
        if expected == 0 && pending_owned_async_declarations.is_empty() {
            if conditional_signal_graph.is_some() {
                return Err(conditional_installation_error(
                    "a conditional Signal graph was supplied without conditional declarations",
                ));
            }
            let Some(bridge) = conditional_runtime_bridge else {
                return Ok((None, Default::default(), Vec::new()));
            };
            let resources = resources.ok_or_else(|| {
                conditional_installation_error(
                    "Product World installation requires explicit conditional_execution_resources(...) limits",
                )
            })?;
            let assembly =
                worth_runtime_bridge::facade::BridgeConditionalRuntimeBuilder::with_owned_signal_graph(
                    bridge,
                    resources.signal_evaluations(),
                )
                .map_err(|denial| {
                    conditional_installation_error(format!(
                        "{:?}: {}",
                        denial.kind(),
                        denial.detail()
                    ))
                })?
                .seal()
                .map_err(|denial| {
                    conditional_installation_error(format!(
                        "{:?}: {}",
                        denial.kind(),
                        denial.detail()
                    ))
                })?;
            return Ok((Some(assembly), Default::default(), Vec::new()));
        }
        let resources = resources.ok_or_else(|| {
            conditional_installation_error(
                "conditional declarations require explicit conditional_execution_resources(...) limits",
            )
        })?;
        let bridge = conditional_runtime_bridge.ok_or_else(|| {
            conditional_installation_error(
                "conditional declarations require the exact Runtime Bridge selected by runtime_bridge(...)"
            )
        })?;
        let requires_external_graph = pending_conditional_installations
            .iter()
            .any(|pending| pending.requires_external_signal_graph());
        let mut signal = match (requires_external_graph, conditional_signal_graph) {
            (true, Some(graph)) => {
                worth_runtime_bridge::facade::BridgeConditionalRuntimeBuilder::new(
                    bridge,
                    graph,
                    resources.signal_evaluations(),
                )
            }
            (true, None) => {
                return Err(conditional_installation_error(
                    "conditional declarations with exact targets require one owned Signal graph",
                ));
            }
            (false, None) => {
                worth_runtime_bridge::facade::BridgeConditionalRuntimeBuilder::with_owned_signal_graph(
                    bridge,
                    resources.signal_evaluations(),
                )
            }
            (false, Some(_)) => {
                return Err(conditional_installation_error(
                    "owned-topology conditional declarations reject caller-supplied Signal graphs",
                ));
            }
        }
        .map_err(|denial| {
            conditional_installation_error(format!("{:?}: {}", denial.kind(), denial.detail()))
        })?;
        let mut installed =
            crate::domain_installation::WorthQueryConditionalExecutionRegistry::default();
        for pending in pending_conditional_installations {
            pending
                .install(domains, graphs, &mut signal, &mut installed)
                .map_err(|denial| conditional_installation_error(format!("{denial:?}")))?;
        }
        let mut installed_async = Vec::with_capacity(pending_owned_async_declarations.len());
        for declaration in pending_owned_async_declarations {
            let lowered = signal
                .install_owned_async_request_response(
                    worth_runtime_bridge::facade::BridgeOwnedAsyncRequestResponseDeclaration::new(
                        declaration.identity().canonical_identity(),
                        declaration.payload_contract(),
                        declaration.max_payload_bytes(),
                        declaration.retry_max_attempts(),
                        declaration.retry_delay_ticks(),
                        declaration.timeout_ticks(),
                    ),
                )
                .map_err(|denial| {
                    conditional_installation_error(format!(
                        "{:?}: {}",
                        denial.kind(),
                        denial.detail()
                    ))
                })?;
            installed_async.push((declaration.clone(), lowered));
        }
        let expected_installed = pending_conditional_installations
            .iter()
            .map(|pending| pending.installed_node_count())
            .sum::<usize>();
        if installed.registration_len() != expected_installed {
            return Err(conditional_installation_error(
                "conditional registration set did not converge on the declared node set",
            ));
        }
        let assembly = signal.seal().map_err(|denial| {
            conditional_installation_error(format!("{:?}: {}", denial.kind(), denial.detail()))
        })?;
        Ok((Some(assembly), installed, installed_async))
    }

    pub(crate) fn prepare_conditional_reconstitution(
        current: &worth_runtime_bridge::facade::BridgeSealedRuntimeAssembly,
        selected: &worth_query_execution::facade::primary_graph::WorthQueryProductBranchLease,
        installed: &crate::domain_installation::WorthQueryConditionalExecutionRegistry,
        domains: &crate::domain_installation::WorthQueryDomainInstallationRegistry,
    ) -> Result<
        (
            worth_runtime_bridge::facade::BridgePreparedConditionalReconstitution,
            crate::domain_installation::WorthQueryConditionalExecutionRegistry,
        ),
        crate::runtime::WorthQueryRuntimeError,
    > {
        let candidate = selected
            .prepare_conditional_reconstitution(current)
            .map_err(|denial| {
                conditional_installation_error(format!("{:?}: {}", denial.kind(), denial.detail()))
            })?;
        let installed = installed
            .prepare_runtime_reconstitution(domains, &candidate)
            .map_err(|denial| conditional_installation_error(format!("{denial:?}")))?;
        Ok((candidate, installed))
    }
}

fn conditional_installation_error(
    message: impl Into<String>,
) -> crate::runtime::WorthQueryRuntimeError {
    crate::runtime::WorthQueryRuntimeError::InvariantRegistration {
        stage: "conditional_node_installation",
        message: message.into(),
    }
}
