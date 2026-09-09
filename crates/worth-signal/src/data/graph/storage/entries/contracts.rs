use crate::data::error::SignalError;
use crate::data::graph::signal_graph::SignalGraph;
use crate::data::handle::NodeId;
use crate::data::node::NodeContract;

impl SignalGraph {
    pub fn get_contract(&self, id: NodeId) -> Result<&NodeContract, SignalError> {
        Ok(&self.definition_ref(id)?.eval_config.contract)
    }

    pub fn node_schema_binding(
        &self,
        id: NodeId,
    ) -> Result<Option<&crate::schema::data::SignalSchemaBinding>, SignalError> {
        Ok(self.definition_ref(id)?.eval_config.schema_binding.as_ref())
    }

    pub fn node_merge_strategy_name(
        &self,
        id: NodeId,
    ) -> Result<Option<&crate::logic::transaction::MergeStrategyName>, SignalError> {
        Ok(self
            .definition_ref(id)?
            .eval_config
            .merge_strategy_name
            .as_ref())
    }

    pub fn node_conflict_policy_name(
        &self,
        id: NodeId,
    ) -> Result<Option<&crate::logic::transaction::ConflictPolicyName>, SignalError> {
        Ok(self
            .definition_ref(id)?
            .eval_config
            .conflict_policy_name
            .as_ref())
    }

    pub fn node_identity_matcher_name(
        &self,
        id: NodeId,
    ) -> Result<Option<&crate::logic::transaction::IdentityMatcherName>, SignalError> {
        Ok(self
            .definition_ref(id)?
            .eval_config
            .identity_matcher_name
            .as_ref())
    }

    pub fn node_source_only_policy_name(
        &self,
        id: NodeId,
    ) -> Result<Option<&crate::logic::transaction::SourceOnlyPolicyName>, SignalError> {
        Ok(self
            .definition_ref(id)?
            .eval_config
            .source_only_policy_name
            .as_ref())
    }

    pub fn node_deletion_policy_name(
        &self,
        id: NodeId,
    ) -> Result<Option<&crate::logic::transaction::DeletionPolicyName>, SignalError> {
        Ok(self
            .definition_ref(id)?
            .eval_config
            .deletion_policy_name
            .as_ref())
    }

    pub fn node_conflict_isolation_policy_name(
        &self,
        id: NodeId,
    ) -> Result<Option<&crate::logic::transaction::ConflictIsolationPolicyName>, SignalError> {
        Ok(self
            .definition_ref(id)?
            .eval_config
            .conflict_isolation_policy_name
            .as_ref())
    }

    pub fn node_aspect_merge_policy_bindings(
        &self,
        id: NodeId,
    ) -> Result<&[crate::logic::transaction::AspectMergePolicyBinding], SignalError> {
        Ok(&self
            .definition_ref(id)?
            .eval_config
            .aspect_merge_policy_bindings)
    }

    pub fn validate_schema_bindings_against(
        &self,
        schema_registry: &crate::schema::data::SignalSchemaRegistry,
    ) -> Result<(), SignalError> {
        for node in self.live_node_ids() {
            let Some(binding) = self.node_schema_binding(node)? else {
                continue;
            };
            let descriptor = schema_registry
                .resolve_by_id(binding.schema_id())
                .ok_or_else(|| {
                    SignalError::invalid_input(format!(
                        "node {} references unknown schema id `{}`",
                        node,
                        binding.schema_id().0
                    ))
                })?;
            if descriptor.semantic_name() != binding.semantic_name() {
                return Err(SignalError::invalid_input(format!(
                    "node {} schema binding name mismatch: binding=`{}`, registry=`{}`",
                    node,
                    binding.semantic_name().as_str(),
                    descriptor.semantic_name().as_str()
                )));
            }
            if descriptor.version() != binding.version() {
                return Err(SignalError::invalid_input(format!(
                    "node {} schema binding version mismatch for `{}`",
                    node,
                    binding.semantic_name().as_str()
                )));
            }
            if descriptor.digest() != binding.descriptor_digest() {
                return Err(SignalError::invalid_input(format!(
                    "node {} schema binding digest mismatch for `{}`",
                    node,
                    binding.semantic_name().as_str()
                )));
            }
        }
        Ok(())
    }

    pub fn validate_merge_semantics_against(
        &self,
        schema_registry: &crate::schema::data::SignalSchemaRegistry,
        merge_strategy_registry: &crate::logic::transaction::FrozenMergeStrategyRegistry,
        aspect_merge_policy_registry: &crate::logic::transaction::FrozenAspectMergePolicyRegistry,
        conflict_isolation_registry: &crate::logic::transaction::FrozenConflictIsolationRegistry,
        conflict_policy_registry: &crate::logic::transaction::FrozenConflictPolicyRegistry,
        identity_matcher_registry: &crate::logic::transaction::FrozenIdentityMatcherRegistry,
        source_only_policy_registry: &crate::logic::transaction::FrozenSourceOnlyPolicyRegistry,
        deletion_policy_registry: &crate::logic::transaction::FrozenDeletionPolicyRegistry,
    ) -> Result<(), SignalError> {
        for registration in schema_registry.iter() {
            let descriptor = registration.descriptor();
            if let Some(strategy_name) = descriptor.default_merge_strategy_name() {
                if merge_strategy_registry
                    .resolve_by_name(strategy_name)
                    .is_none()
                {
                    return Err(SignalError::invalid_input(format!(
                        "schema `{}` references unknown default merge strategy `{}`",
                        descriptor.semantic_name().as_str(),
                        strategy_name.as_str()
                    )));
                }
            }
            if let Some(policy_name) = descriptor.default_conflict_policy_name() {
                if conflict_policy_registry
                    .resolve_by_name(policy_name)
                    .is_none()
                {
                    return Err(SignalError::invalid_input(format!(
                        "schema `{}` references unknown default conflict policy `{}`",
                        descriptor.semantic_name().as_str(),
                        policy_name.as_str()
                    )));
                }
            }
            if let Some(matcher_name) = descriptor.default_identity_matcher_name() {
                if identity_matcher_registry
                    .resolve_by_name(matcher_name)
                    .is_none()
                {
                    return Err(SignalError::invalid_input(format!(
                        "schema `{}` references unknown default identity matcher `{}`",
                        descriptor.semantic_name().as_str(),
                        matcher_name.as_str()
                    )));
                }
            }
            if let Some(policy_name) = descriptor.default_source_only_policy_name() {
                if source_only_policy_registry
                    .resolve_by_name(policy_name)
                    .is_none()
                {
                    return Err(SignalError::invalid_input(format!(
                        "schema `{}` references unknown default source-only policy `{}`",
                        descriptor.semantic_name().as_str(),
                        policy_name.as_str()
                    )));
                }
            }
            if let Some(policy_name) = descriptor.default_deletion_policy_name() {
                if deletion_policy_registry
                    .resolve_by_name(policy_name)
                    .is_none()
                {
                    return Err(SignalError::invalid_input(format!(
                        "schema `{}` references unknown default deletion policy `{}`",
                        descriptor.semantic_name().as_str(),
                        policy_name.as_str()
                    )));
                }
            }
            if let Some(policy_name) = descriptor.default_conflict_isolation_policy_name() {
                if conflict_isolation_registry
                    .resolve_by_name(policy_name)
                    .is_none()
                {
                    return Err(SignalError::invalid_input(format!(
                        "schema `{}` references unknown default conflict isolation policy `{}`",
                        descriptor.semantic_name().as_str(),
                        policy_name.as_str()
                    )));
                }
            }
            for binding in descriptor.default_aspect_merge_policy_bindings() {
                if aspect_merge_policy_registry
                    .resolve_by_name(&binding.policy_name)
                    .is_none()
                {
                    return Err(SignalError::invalid_input(format!(
                        "schema `{}` references unknown aspect merge policy `{}` for aspect {}",
                        descriptor.semantic_name().as_str(),
                        binding.policy_name.as_str(),
                        binding.aspect.id()
                    )));
                }
            }
        }

        for node in self.live_node_ids() {
            if let Some(strategy_name) = self.node_merge_strategy_name(node)? {
                if merge_strategy_registry
                    .resolve_by_name(strategy_name)
                    .is_none()
                {
                    return Err(SignalError::invalid_input(format!(
                        "node {} references unknown merge strategy `{}`",
                        node,
                        strategy_name.as_str()
                    )));
                }
            }
            if let Some(policy_name) = self.node_conflict_policy_name(node)? {
                if conflict_policy_registry
                    .resolve_by_name(policy_name)
                    .is_none()
                {
                    return Err(SignalError::invalid_input(format!(
                        "node {} references unknown conflict policy `{}`",
                        node,
                        policy_name.as_str()
                    )));
                }
            }
            if let Some(matcher_name) = self.node_identity_matcher_name(node)? {
                if identity_matcher_registry
                    .resolve_by_name(matcher_name)
                    .is_none()
                {
                    return Err(SignalError::invalid_input(format!(
                        "node {} references unknown identity matcher `{}`",
                        node,
                        matcher_name.as_str()
                    )));
                }
            }
            if let Some(policy_name) = self.node_source_only_policy_name(node)? {
                if source_only_policy_registry
                    .resolve_by_name(policy_name)
                    .is_none()
                {
                    return Err(SignalError::invalid_input(format!(
                        "node {} references unknown source-only policy `{}`",
                        node,
                        policy_name.as_str()
                    )));
                }
            }
            if let Some(policy_name) = self.node_deletion_policy_name(node)? {
                if deletion_policy_registry
                    .resolve_by_name(policy_name)
                    .is_none()
                {
                    return Err(SignalError::invalid_input(format!(
                        "node {} references unknown deletion policy `{}`",
                        node,
                        policy_name.as_str()
                    )));
                }
            }
            if let Some(policy_name) = self.node_conflict_isolation_policy_name(node)? {
                if conflict_isolation_registry
                    .resolve_by_name(policy_name)
                    .is_none()
                {
                    return Err(SignalError::invalid_input(format!(
                        "node {} references unknown conflict isolation policy `{}`",
                        node,
                        policy_name.as_str()
                    )));
                }
            }
            for binding in self.node_aspect_merge_policy_bindings(node)? {
                if aspect_merge_policy_registry
                    .resolve_by_name(&binding.policy_name)
                    .is_none()
                {
                    return Err(SignalError::invalid_input(format!(
                        "node {} references unknown aspect merge policy `{}` for aspect {}",
                        node,
                        binding.policy_name.as_str(),
                        binding.aspect.id()
                    )));
                }
            }
        }

        Ok(())
    }
}
