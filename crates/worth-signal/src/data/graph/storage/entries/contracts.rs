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
}
