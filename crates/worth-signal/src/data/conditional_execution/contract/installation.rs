use super::*;

impl SignalGraph {
    pub fn install_conditional_contract(
        &mut self,
        owner: &SignalAspectLoweringOwner,
        node: InstalledSignalNodeCapability,
        definition: SignalConditionalContractDefinition,
    ) -> Result<InstalledSignalConditionalContract, SignalConditionalContractDenial> {
        let prepared = self.prepare_conditional_contract(owner, node, definition)?;
        self.apply_prepared_conditional_contract(prepared)
    }

    pub(crate) fn prepare_conditional_contract(
        &self,
        owner: &SignalAspectLoweringOwner,
        node: InstalledSignalNodeCapability,
        definition: SignalConditionalContractDefinition,
    ) -> Result<PreparedSignalConditionalContract, SignalConditionalContractDenial> {
        if node.graph_instance_id() != self.runtime_instance_id() {
            return Err(SignalConditionalContractDenial::ForeignGraph);
        }
        if !self
            .aspect_lowering_owner
            .as_ref()
            .is_some_and(|installed| installed.is_same_owner(owner))
        {
            return Err(SignalConditionalContractDenial::ForeignLoweringOwner);
        }
        self.get_contract(node.node())
            .map_err(|_| SignalConditionalContractDenial::StaleNode)?;
        let entry = self
            .get_entry(node.node())
            .map_err(|_| SignalConditionalContractDenial::StaleNode)?;
        let predecessor_generation = entry.conditional_contract_generation();
        let predecessor_occurrence = entry.conditional_contract_occurrence();
        let generation = predecessor_generation
            .checked_add(1)
            .ok_or(SignalConditionalContractDenial::GenerationExhausted)?;
        let occurrence = NEXT_CONDITIONAL_CONTRACT_OCCURRENCE
            .fetch_update(
                std::sync::atomic::Ordering::Relaxed,
                std::sync::atomic::Ordering::Relaxed,
                |value| value.checked_add(1),
            )
            .map_err(|_| SignalConditionalContractDenial::OccurrenceExhausted)?;
        let service_retained_bytes = definition.installation_retained_bytes();
        let semantic_condition = definition.condition;
        let condition =
            lower_condition(self.runtime_instance_id(), node.node(), &semantic_condition);
        let dependency_comparator = lower_comparator(
            self.runtime_instance_id(),
            node.node(),
            InstalledSignalComparatorRole::DependencyVersion,
            definition.dependency_comparator,
        );
        let output_comparator = lower_comparator(
            self.runtime_instance_id(),
            node.node(),
            InstalledSignalComparatorRole::OutputEquivalence,
            definition.output_comparator,
        );
        let output_equivalence =
            OutputEquivalencePolicy::from_installed_comparator(output_comparator.clone())
                .expect("output comparator lowering must retain the output-equivalence role");
        let artifact_reuse = lower_artifact_reuse(
            self.runtime_instance_id(),
            node.node(),
            definition.artifact_reuse,
        );
        let mut config = self
            .node_eval_config(node.node())
            .map_err(|_| SignalConditionalContractDenial::StaleNode)?
            .clone();
        config.condition = condition.clone();
        config.comparator = Some(dependency_comparator.clone());
        config.output_equivalence = output_equivalence.clone();
        config.contract = config.contract.with_output_equivalence(&output_equivalence);
        let mut installed = InstalledSignalConditionalContract {
            authority: std::sync::Arc::new(InstalledSignalConditionalAuthority { _owner_seal: () }),
            graph_instance_id: self.runtime_instance_id(),
            node: node.node(),
            generation,
            occurrence,
            condition,
            semantic_condition,
            dependency_aspects: definition.dependency_aspects,
            trigger_aspects: definition.trigger_aspects,
            dependency_comparator,
            output_comparator,
            output_equivalence,
            artifact_reuse,
            projection_contract: String::new(),
            service_retained_bytes,
        };
        installed.projection_contract =
            crate::data::conditional_execution::identity::contract_projection_basis(&installed);
        Ok(PreparedSignalConditionalContract {
            node: node.node(),
            predecessor_generation,
            predecessor_occurrence,
            config,
            contract: installed,
        })
    }

    pub(crate) fn apply_prepared_conditional_contract(
        &mut self,
        prepared: PreparedSignalConditionalContract,
    ) -> Result<InstalledSignalConditionalContract, SignalConditionalContractDenial> {
        let entry = self
            .get_entry(prepared.node)
            .map_err(|_| SignalConditionalContractDenial::StaleNode)?;
        if entry.conditional_contract_generation() != prepared.predecessor_generation
            || entry.conditional_contract_occurrence() != prepared.predecessor_occurrence
        {
            return Err(SignalConditionalContractDenial::StaleNode);
        }
        install_node_evaluation_config(self, prepared.node, prepared.config)?;
        let installed_generation = self
            .get_entry_mut(prepared.node)
            .map_err(|_| SignalConditionalContractDenial::StaleNode)?
            .install_conditional_contract_occurrence(prepared.contract.occurrence)
            .expect("prepared generation was checked before installation");
        debug_assert_eq!(installed_generation, prepared.contract.generation);
        Ok(prepared.contract)
    }
}
