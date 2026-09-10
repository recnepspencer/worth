use super::*;

impl WorthQueryRuntimeBuilder {
    pub fn build(mut self) -> Result<WorthQueryRuntime, WorthQueryRuntimeError> {
        self.queue_installed_domain_substrates();
        let mut conditional_runtime_bridge = self.conditional_runtime_bridge.take();
        let conditional_signal_graph = self.conditional_signal_graph.take();
        let conditional_execution_resources = self.conditional_execution_resources.take();
        let pending_conditional_installations =
            std::mem::take(&mut self.pending_conditional_installations);
        let pending_owned_async_declarations =
            std::mem::take(&mut self.pending_owned_async_declarations);
        let pending_primary_graph_installation = self.pending_primary_graph_installation.take();
        let product_world_resources = self.product_world_resources.clone();
        if self.backend.is_some()
            && (!self.backend_parts.is_empty() || self.pending_relational_product_bridge.is_some())
        {
            return Err(WorthQueryRuntimeError::InvariantRegistration {
                stage: "runtime_backend_authority_selection",
                message: "explicit runtime backends cannot be combined with backend parts such as runtime_bridge(...), schema_adapter(...), or write_authority(...), or relational_product_bridge(...); choose one backend authority path".to_string(),
            });
        }
        if self.backend.is_some() && !self.queued_invariant_registrations.is_empty() {
            return Err(WorthQueryRuntimeError::InvariantRegistration {
                stage: "runtime_backend_selection",
                message: "queued Query-owned invariant registrations cannot be applied after selecting an explicit runtime backend; lower them through WorthQueryRuntimeBuilder before backend(...) or relational_runtime(...)".to_string(),
            });
        }
        let mut backend = self
            .backend
            .ok_or(WorthQueryRuntimeError::MissingBackend)??;
        let primary_runtime_invalidation_installation =
            self.primary_runtime_invalidation_installation.take();
        if let Some(primary) = primary_runtime_invalidation_installation.as_ref() {
            if !backend.readmits_primary_graph_source(primary) {
                return Err(WorthQueryRuntimeError::Workspace(
                    WorthQueryWorkspaceError::new(
                        "primary granular invalidation and source adapters do not retain the same current runtime",
                    ),
                ));
            }
            backend
                .attach_primary_graph_runtime(WorthQueryPrimaryGraphBackendHandle::new(
                    primary.retain_primary_graph_integration_handle(),
                ))
                .map_err(WorthQueryRuntimeError::Workspace)?;
        }
        let consumer_support_profile =
            crate::domain_installation::WorthQueryConsumerSupportProfile::from_runtime(
                &backend.support_profile(),
            )
            .with_runtime_overrides(self.consumer_support_postures);
        let installed_domain_artifacts = self.pending_domain_installations.into_artifacts();
        let execution_runtime_installation = match self.host_execution_installation.take() {
            Some(installation) => installation,
            None => {
                let installer =
                    worth_query_execution::facade::runtime::WorthQueryExecutionRuntimeInstaller::new();
                installer
                    .install(
                        worth_query_installation::facade::WorthQueryInstallationGeneration::initial(
                        ),
                        installed_domain_artifacts
                            .iter()
                            .map(|artifact| artifact.portable_package.clone()),
                    )
                    .expect("locally admitted packages must build the execution installed index")
            }
        };
        let (mut execution_runtime, execution_installation_authority) =
            execution_runtime_installation.into_parts();
        let authority_identity = execution_runtime.authority_identity();
        let graph_participation_registry = self
            .pending_graph_participations
            .install(
                authority_identity,
                execution_runtime
                    .installed_packages()
                    .installation_runtime(),
            )
            .map_err(|denial| WorthQueryRuntimeError::InvariantRegistration {
                stage: "graph_participation_installation",
                message: format!("{:?}: {}", denial.kind(), denial.detail()),
            })?;
        let primary_graph_installation = pending_primary_graph_installation
            .map(|pending| {
                pending.install(
                    &mut execution_runtime,
                    &execution_installation_authority,
                    backend.as_mut(),
                    product_world_resources,
                )
            })
            .transpose()?;
        let primary_graph_publication = primary_graph_installation.map(|installed| {
            conditional_runtime_bridge = Some(installed.product_bridge);
            installed.publication
        });
        let domain_installation_registry =
            crate::domain_installation::WorthQueryDomainInstallationRegistry::from_artifacts(
                installed_domain_artifacts,
                authority_identity,
                execution_runtime.retain_installed_packages(),
            );
        let (
            conditional_signal_runtime,
            conditional_execution_registry,
            installed_owned_async_declarations,
        ) = WorthQueryRuntimeBuilder::install_conditional_execution(
            conditional_runtime_bridge,
            conditional_signal_graph,
            conditional_execution_resources,
            &pending_conditional_installations,
            &pending_owned_async_declarations,
            &domain_installation_registry,
            &graph_participation_registry,
        )?;
        let conditional = conditional_signal_runtime.ok_or_else(|| {
            WorthQueryRuntimeError::InvariantRegistration {
                stage: "product_world_installation",
                message: "execution runtime construction requires the exact Runtime Bridge and explicit Product World resources"
                    .to_owned(),
            }
        })?;
        let resources = conditional_execution_resources
            .expect("sealed conditional runtime installation requires its resource configuration");
        let installed_product =
            super::super::installed_product::WorthQueryInstalledProduct::install(
                backend.as_ref(),
                conditional,
                resources.query_cache(),
                self.product_world_resources,
            )?;
        let installed_owned_async_declarations =
            super::super::owned_async_source::install_owned_async_registry(
                authority_identity,
                &installed_product,
                installed_owned_async_declarations,
            )?;
        let domain_operation_executor_registry = self
            .pending_domain_operation_executors
            .install(
                &domain_installation_registry
                    .execution_index()
                    .domain_operation_execution_descriptors(),
            )
            .map_err(|message| WorthQueryRuntimeError::InvariantRegistration {
                stage: "domain_operation_executor_installation",
                message: message.to_string(),
            })?;
        let workflow_descriptors = domain_installation_registry
            .execution_index()
            .workflow_operation_execution_descriptors();
        let workflow_stage_executor_registry = self
            .pending_workflow_stage_executors
            .install(&workflow_descriptors)
            .map_err(|message| WorthQueryRuntimeError::InvariantRegistration {
                stage: "workflow_stage_executor_installation",
                message: message.to_string(),
            })?;
        let workflow_parallel_admission_provider_registry = self
            .pending_workflow_parallel_admission_providers
            .install(&workflow_descriptors)
            .map_err(|message| WorthQueryRuntimeError::InvariantRegistration {
                stage: "workflow_parallel_admission_provider_installation",
                message: message.to_string(),
            })?;
        let native_aspect_contracts = self.native_aspect_contracts;
        Ok(WorthQueryRuntime {
            backend,
            evidence_authority: WorthQueryRuntimeEvidenceAuthority::new(),
            authority_identity,
            execution_runtime,
            execution_installation_authority,
            primary_graph_publication,
            primary_runtime_invalidation_installation,
            domain_installation_registry,
            domain_operation_executor_registry,
            workflow_stage_executor_registry,
            workflow_parallel_admission_provider_registry,
            graph_participation_registry,
            installed_product,
            conditional_execution_registry,
            installed_owned_async_declarations,
            installed_live_routes: Default::default(),
            shared_projection_owners: Default::default(),
            consumer_support_profile,
            native_aspect_contracts,
            preview_session_labels: BTreeSet::new(),
            branch_session_labels: BTreeSet::new(),
            active_subscriptions: ActiveSubscriptionRuntime::new(),
            live_subscriptions: BTreeMap::new(),
            granular_projection_states: BTreeMap::new(),
            materialized_read_views: BTreeMap::new(),
            live_subscription_index: Default::default(),
            installed_programs: BTreeMap::new(),
            run_traces: BTreeMap::new(),
            derived_views: BTreeMap::new(),
            shared_read_pins:
                super::super::shared_read_pins::WorthQuerySharedReadPinRegistry::default(),
            published_artifacts:
                super::super::published_artifacts::WorthQueryPublishedArtifactRegistry::default(),
            journal_replay: super::super::journal_replay::WorthQueryJournalReplayRegistry::default(
            ),
            derived_dependency_index: WorthQueryComputedDependencyIndex::default(),
            effects: BTreeMap::new(),
            effect_index: WorthQueryEffectIndex::default(),
            managed_live_resource_capability: WorthQueryManagedLiveWorkspaceCapability::shared(),
            next_run_id: 0,
        })
    }
}
