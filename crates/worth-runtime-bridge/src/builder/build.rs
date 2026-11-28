use super::*;

fn finalize_source_configuration(
    source_declarations: Vec<SourceDeclaration>,
    mut source_adapter_registrations: Vec<Arc<dyn BridgeSourceAdapter>>,
) -> Result<(AdmittedSourceRegistry, Option<Arc<dyn BridgeSourceAdapter>>), BridgeBuildError> {
    if source_adapter_registrations.len() > 1 {
        return Err(BridgeBuildError::new(
            BridgeBuildErrorKind::BuilderConfigurationConflict,
            "Bridge builder registered more than one source adapter for the same runtime.",
        ));
    }

    let source_registry = AdmittedSourceRegistry::freeze(source_declarations)?;
    let source_adapter = source_adapter_registrations.pop();

    if !source_registry.contracts().is_empty() && source_adapter.is_none() {
        return Err(BridgeBuildError::new(
            BridgeBuildErrorKind::MissingSourceAdapter,
            "Bridge builder registered source declarations but no source adapter.",
        ));
    }

    if let Some(adapter) = source_adapter.as_ref() {
        let declared_capabilities = adapter.declared_capabilities();
        let required_capabilities = source_registry.required_capabilities();
        if !declared_capabilities.contains_all(&required_capabilities) {
            return Err(BridgeBuildError::new(
                BridgeBuildErrorKind::SourceCapabilityMismatch,
                format!(
                    "Bridge source adapter capabilities `{}` do not satisfy required source capabilities `{}`.",
                    declared_capabilities.digest(),
                    required_capabilities.digest()
                ),
            ));
        }
    }

    Ok((source_registry, source_adapter))
}

fn finalize_structural_configuration(
    structural_declarations: Vec<StructuralIdentityDeclaration>,
) -> Result<AdmittedStructuralRegistry, BridgeBuildError> {
    AdmittedStructuralRegistry::freeze(structural_declarations)
}

impl
    RuntimeBridgeBuilder<
        PresentCommittedPatchSource,
        PresentSnapshotReadSource,
        PresentSignalSink,
        MissingTruthBranchHeadSource,
        PresentMappingRegistrations,
    >
{
    pub fn build(self) -> Result<RuntimeBridge, BridgeBuildError> {
        let mapping_registry = FrozenMappingRegistry::freeze(self.mapping_registrations.0)?;
        let aspect_registry = FrozenAspectMappingRegistry::freeze(self.aspect_registrations)?;
        let subscription_family_registry =
            crate::subscription::freeze_subscription_family_registry()?;
        let semantic_dependency_registry =
            AdmittedSemanticDependencyRegistry::freeze(self.semantic_dependency_registrations)?;
        let policy = super::policy::finalize_runtime_policy(self.policy);
        let (source_registry, source_adapter) = finalize_source_configuration(
            self.source_declarations,
            self.source_adapter_registrations,
        )?;
        let structural_registry = finalize_structural_configuration(self.structural_declarations)?;
        Ok(RuntimeBridge::new(
            policy,
            self.diagnostic_sink,
            self.committed_patch_source.0,
            self.snapshot_read_source.0,
            self.signal_sink.0,
            None,
            self.continuity_lineage_source,
            self.writeback_authority,
            self.snapshot_reader_pool,
            source_registry,
            source_adapter,
            structural_registry,
            mapping_registry,
            aspect_registry,
            subscription_family_registry,
            semantic_dependency_registry,
        ))
    }
}

impl
    RuntimeBridgeBuilder<
        PresentCommittedPatchSource,
        PresentSnapshotReadSource,
        PresentSignalSink,
        PresentTruthBranchHeadSource,
        PresentMappingRegistrations,
    >
{
    pub fn build(self) -> Result<RuntimeBridge, BridgeBuildError> {
        let mapping_registry = FrozenMappingRegistry::freeze(self.mapping_registrations.0)?;
        let aspect_registry = FrozenAspectMappingRegistry::freeze(self.aspect_registrations)?;
        let subscription_family_registry =
            crate::subscription::freeze_subscription_family_registry()?;
        let semantic_dependency_registry =
            AdmittedSemanticDependencyRegistry::freeze(self.semantic_dependency_registrations)?;
        let policy = super::policy::finalize_runtime_policy(self.policy);
        let (source_registry, source_adapter) = finalize_source_configuration(
            self.source_declarations,
            self.source_adapter_registrations,
        )?;
        let structural_registry = finalize_structural_configuration(self.structural_declarations)?;
        Ok(RuntimeBridge::new(
            policy,
            self.diagnostic_sink,
            self.committed_patch_source.0,
            self.snapshot_read_source.0,
            self.signal_sink.0,
            Some(self.truth_branch_head_source.0),
            self.continuity_lineage_source,
            self.writeback_authority,
            self.snapshot_reader_pool,
            source_registry,
            source_adapter,
            structural_registry,
            mapping_registry,
            aspect_registry,
            subscription_family_registry,
            semantic_dependency_registry,
        ))
    }
}
