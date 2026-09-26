use crate::domain_computation::authorization::WorthQueryOperationScopeBinding;

#[cfg(test)]
mod tests;
mod workflow_definition;
mod workflow_instance;
mod workflow_proposal_context;
mod workflow_transition;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
struct WorthQueryIdempotencyEntityIdentity {
    partition: u32,
    local_slot: u64,
    generation: u32,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
struct WorthQueryIdempotencyScopeIdentity {
    runtime_authority: u64,
    binding_runtime: u64,
    binding_generation: u64,
    package_identity: [u8; 32],
    schema_identity: [u8; 32],
    principal: WorthQueryIdempotencyEntityIdentity,
    scope: WorthQueryIdempotencyEntityIdentity,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct WorthQueryApplicationIdempotencyBinding {
    key_identity: [u8; 32],
    intent_identity: [u8; 32],
    source_identity: Option<[u8; 32]>,
    source_partition_identity: Option<[u8; 32]>,
    producer_dependency_identity: Option<[u8; 32]>,
    operation_identity: Option<[u8; 32]>,
    operation_scope_identity: Option<WorthQueryIdempotencyScopeIdentity>,
    precondition_identity: Option<[u8; 32]>,
    governed_input_identity: Option<[u8; 32]>,
    governed_proposal_identity: Option<[u8; 32]>,
    conditional_definition_identity: Option<[u8; 32]>,
    workflow_definition_identity: Option<[u8; 32]>,
    workflow_instance_identity: Option<[u8; 32]>,
    workflow_proposal_context_identity: Option<[u8; 32]>,
    workflow_transition_identity: Option<[u8; 32]>,
    workflow_support_identity: Option<[u8; 32]>,
    workflow_client_key_identity: Option<[u8; 32]>,
    workflow_approval_identity: Option<[u8; 32]>,
}

impl WorthQueryApplicationIdempotencyBinding {
    pub const fn new(key_identity: [u8; 32], intent_identity: [u8; 32]) -> Self {
        Self {
            key_identity,
            intent_identity,
            source_identity: None,
            source_partition_identity: None,
            producer_dependency_identity: None,
            operation_identity: None,
            operation_scope_identity: None,
            precondition_identity: None,
            governed_input_identity: None,
            governed_proposal_identity: None,
            conditional_definition_identity: None,
            workflow_definition_identity: None,
            workflow_instance_identity: None,
            workflow_proposal_context_identity: None,
            workflow_transition_identity: None,
            workflow_support_identity: None,
            workflow_client_key_identity: None,
            workflow_approval_identity: None,
        }
    }

    pub const fn key_identity(&self) -> &[u8; 32] {
        &self.key_identity
    }

    pub const fn intent_identity(&self) -> &[u8; 32] {
        &self.intent_identity
    }

    pub(in crate::domain_computation::primary_graph) const fn source_identity(
        &self,
    ) -> Option<[u8; 32]> {
        self.source_identity
    }

    pub(in crate::domain_computation::primary_graph) const fn source_partition_identity(
        &self,
    ) -> Option<[u8; 32]> {
        self.source_partition_identity
    }

    pub(in crate::domain_computation::primary_graph) const fn producer_dependency_identity(
        &self,
    ) -> Option<[u8; 32]> {
        self.producer_dependency_identity
    }

    pub(in crate::domain_computation::primary_graph) fn key_text(self) -> String {
        encode_identity(self.key_identity)
    }

    pub(in crate::domain_computation::primary_graph) fn intent_text(self) -> String {
        let mut encoded = encode_identity(self.intent_identity);
        append_identity_slot(&mut encoded, "source", self.source_identity);
        append_identity_slot(&mut encoded, "operation", self.operation_identity);
        append_scope_slot(&mut encoded, self.operation_scope_identity);
        append_identity_slot(&mut encoded, "precondition", self.precondition_identity);
        append_identity_slot(&mut encoded, "input", self.governed_input_identity);
        append_identity_slot(&mut encoded, "proposal", self.governed_proposal_identity);
        append_identity_slot(
            &mut encoded,
            "conditional-definition",
            self.conditional_definition_identity,
        );
        workflow_definition::append_identity_slot(&mut encoded, self.workflow_definition_identity);
        workflow_instance::append_identity_slot(&mut encoded, self.workflow_instance_identity);
        workflow_proposal_context::append_identity_slot(
            &mut encoded,
            self.workflow_proposal_context_identity,
        );
        workflow_transition::append_identity_slot(&mut encoded, self.workflow_transition_identity);
        workflow_transition::append_support_identity_slot(
            &mut encoded,
            self.workflow_support_identity,
        );
        workflow_transition::append_client_key_identity_slot(
            &mut encoded,
            self.workflow_client_key_identity,
        );
        workflow_transition::append_approval_identity_slot(
            &mut encoded,
            self.workflow_approval_identity,
        );
        encoded
    }

    pub const fn bind_source(mut self, source_identity: Option<&[u8; 32]>) -> Self {
        self.source_identity = match source_identity {
            Some(identity) => Some(*identity),
            None => None,
        };
        self
    }

    pub(in crate::domain_computation::primary_graph) const fn bind_source_partition(
        mut self,
        partition_identity: &[u8; 32],
    ) -> Self {
        self.source_partition_identity = Some(*partition_identity);
        self
    }

    pub(in crate::domain_computation::primary_graph) const fn bind_producer_dependency(
        mut self,
        dependency_identity: &[u8; 32],
    ) -> Self {
        self.producer_dependency_identity = Some(*dependency_identity);
        self
    }

    pub(in crate::domain_computation::primary_graph) const fn bind_operation(
        self,
        operation_identity: &[u8; 32],
    ) -> Self {
        Self {
            key_identity: self.key_identity,
            intent_identity: self.intent_identity,
            source_identity: self.source_identity,
            source_partition_identity: self.source_partition_identity,
            producer_dependency_identity: self.producer_dependency_identity,
            operation_identity: Some(*operation_identity),
            operation_scope_identity: self.operation_scope_identity,
            precondition_identity: self.precondition_identity,
            governed_input_identity: self.governed_input_identity,
            governed_proposal_identity: self.governed_proposal_identity,
            conditional_definition_identity: self.conditional_definition_identity,
            workflow_definition_identity: self.workflow_definition_identity,
            workflow_instance_identity: self.workflow_instance_identity,
            workflow_proposal_context_identity: self.workflow_proposal_context_identity,
            workflow_transition_identity: self.workflow_transition_identity,
            workflow_support_identity: self.workflow_support_identity,
            workflow_client_key_identity: self.workflow_client_key_identity,
            workflow_approval_identity: self.workflow_approval_identity,
        }
    }

    pub(in crate::domain_computation::primary_graph) fn bind_operation_scope(
        self,
        binding: &WorthQueryOperationScopeBinding,
    ) -> Self {
        let principal = binding.principal();
        let scope = binding.scope();
        Self {
            key_identity: self.key_identity,
            intent_identity: self.intent_identity,
            source_identity: self.source_identity,
            source_partition_identity: self.source_partition_identity,
            producer_dependency_identity: self.producer_dependency_identity,
            operation_identity: self.operation_identity,
            operation_scope_identity: Some(WorthQueryIdempotencyScopeIdentity {
                runtime_authority: binding.runtime_authority(),
                binding_runtime: binding.binding_identity().runtime_ordinal(),
                binding_generation: binding.binding_identity().generation(),
                package_identity: *binding.binding_identity().package_identity().bytes(),
                schema_identity: *binding.binding_identity().schema_identity().bytes(),
                principal: WorthQueryIdempotencyEntityIdentity {
                    partition: principal.partition_id(),
                    local_slot: principal.local_slot(),
                    generation: principal.generation(),
                },
                scope: WorthQueryIdempotencyEntityIdentity {
                    partition: scope.partition_id(),
                    local_slot: scope.local_slot(),
                    generation: scope.generation(),
                },
            }),
            precondition_identity: self.precondition_identity,
            governed_input_identity: self.governed_input_identity,
            governed_proposal_identity: self.governed_proposal_identity,
            conditional_definition_identity: self.conditional_definition_identity,
            workflow_definition_identity: self.workflow_definition_identity,
            workflow_instance_identity: self.workflow_instance_identity,
            workflow_proposal_context_identity: self.workflow_proposal_context_identity,
            workflow_transition_identity: self.workflow_transition_identity,
            workflow_support_identity: self.workflow_support_identity,
            workflow_client_key_identity: self.workflow_client_key_identity,
            workflow_approval_identity: self.workflow_approval_identity,
        }
    }

    pub(in crate::domain_computation::primary_graph) const fn bind_preconditions(
        self,
        precondition_identity: Option<&[u8; 32]>,
    ) -> Self {
        Self {
            key_identity: self.key_identity,
            intent_identity: self.intent_identity,
            source_identity: self.source_identity,
            source_partition_identity: self.source_partition_identity,
            producer_dependency_identity: self.producer_dependency_identity,
            operation_identity: self.operation_identity,
            operation_scope_identity: self.operation_scope_identity,
            precondition_identity: match precondition_identity {
                Some(identity) => Some(*identity),
                None => None,
            },
            governed_input_identity: self.governed_input_identity,
            governed_proposal_identity: self.governed_proposal_identity,
            conditional_definition_identity: self.conditional_definition_identity,
            workflow_definition_identity: self.workflow_definition_identity,
            workflow_instance_identity: self.workflow_instance_identity,
            workflow_proposal_context_identity: self.workflow_proposal_context_identity,
            workflow_transition_identity: self.workflow_transition_identity,
            workflow_support_identity: self.workflow_support_identity,
            workflow_client_key_identity: self.workflow_client_key_identity,
            workflow_approval_identity: self.workflow_approval_identity,
        }
    }

    pub(in crate::domain_computation::primary_graph) const fn bind_governed_input(
        self,
        governed_input_identity: Option<&[u8; 32]>,
    ) -> Self {
        Self {
            key_identity: self.key_identity,
            intent_identity: self.intent_identity,
            source_identity: self.source_identity,
            source_partition_identity: self.source_partition_identity,
            producer_dependency_identity: self.producer_dependency_identity,
            operation_identity: self.operation_identity,
            operation_scope_identity: self.operation_scope_identity,
            precondition_identity: self.precondition_identity,
            governed_input_identity: match governed_input_identity {
                Some(identity) => Some(*identity),
                None => None,
            },
            governed_proposal_identity: self.governed_proposal_identity,
            conditional_definition_identity: self.conditional_definition_identity,
            workflow_definition_identity: self.workflow_definition_identity,
            workflow_instance_identity: self.workflow_instance_identity,
            workflow_proposal_context_identity: self.workflow_proposal_context_identity,
            workflow_transition_identity: self.workflow_transition_identity,
            workflow_support_identity: self.workflow_support_identity,
            workflow_client_key_identity: self.workflow_client_key_identity,
            workflow_approval_identity: self.workflow_approval_identity,
        }
    }

    pub(in crate::domain_computation::primary_graph) const fn bind_governed_proposal(
        self,
        governed_proposal_identity: Option<&[u8; 32]>,
    ) -> Self {
        Self {
            key_identity: self.key_identity,
            intent_identity: self.intent_identity,
            source_identity: self.source_identity,
            source_partition_identity: self.source_partition_identity,
            producer_dependency_identity: self.producer_dependency_identity,
            operation_identity: self.operation_identity,
            operation_scope_identity: self.operation_scope_identity,
            precondition_identity: self.precondition_identity,
            governed_input_identity: self.governed_input_identity,
            governed_proposal_identity: match governed_proposal_identity {
                Some(identity) => Some(*identity),
                None => None,
            },
            conditional_definition_identity: self.conditional_definition_identity,
            workflow_definition_identity: self.workflow_definition_identity,
            workflow_instance_identity: self.workflow_instance_identity,
            workflow_proposal_context_identity: self.workflow_proposal_context_identity,
            workflow_transition_identity: self.workflow_transition_identity,
            workflow_support_identity: self.workflow_support_identity,
            workflow_client_key_identity: self.workflow_client_key_identity,
            workflow_approval_identity: self.workflow_approval_identity,
        }
    }

    pub(in crate::domain_computation::primary_graph) const fn bind_conditional_definition(
        self,
        identity: Option<&[u8; 32]>,
    ) -> Self {
        Self {
            key_identity: self.key_identity,
            intent_identity: self.intent_identity,
            source_identity: self.source_identity,
            source_partition_identity: self.source_partition_identity,
            producer_dependency_identity: self.producer_dependency_identity,
            operation_identity: self.operation_identity,
            operation_scope_identity: self.operation_scope_identity,
            precondition_identity: self.precondition_identity,
            governed_input_identity: self.governed_input_identity,
            governed_proposal_identity: self.governed_proposal_identity,
            conditional_definition_identity: match identity {
                Some(identity) => Some(*identity),
                None => None,
            },
            workflow_definition_identity: self.workflow_definition_identity,
            workflow_instance_identity: self.workflow_instance_identity,
            workflow_proposal_context_identity: self.workflow_proposal_context_identity,
            workflow_transition_identity: self.workflow_transition_identity,
            workflow_support_identity: self.workflow_support_identity,
            workflow_client_key_identity: self.workflow_client_key_identity,
            workflow_approval_identity: self.workflow_approval_identity,
        }
    }
}

fn append_identity_slot(encoded: &mut String, slot: &str, identity: Option<[u8; 32]>) {
    encoded.push(':');
    encoded.push_str(slot);
    encoded.push('=');
    match identity {
        Some(identity) => append_bytes(encoded, &identity),
        None => encoded.push('-'),
    }
}

fn append_scope_slot(encoded: &mut String, identity: Option<WorthQueryIdempotencyScopeIdentity>) {
    encoded.push_str(":scope=");
    let Some(identity) = identity else {
        encoded.push('-');
        return;
    };
    append_bytes(encoded, &identity.runtime_authority.to_be_bytes());
    append_bytes(encoded, &identity.binding_runtime.to_be_bytes());
    append_bytes(encoded, &identity.binding_generation.to_be_bytes());
    append_bytes(encoded, &identity.package_identity);
    append_bytes(encoded, &identity.schema_identity);
    append_entity_identity(encoded, identity.principal);
    append_entity_identity(encoded, identity.scope);
}

fn append_entity_identity(encoded: &mut String, identity: WorthQueryIdempotencyEntityIdentity) {
    append_bytes(encoded, &identity.partition.to_be_bytes());
    append_bytes(encoded, &identity.local_slot.to_be_bytes());
    append_bytes(encoded, &identity.generation.to_be_bytes());
}

fn encode_identity(identity: [u8; 32]) -> String {
    let mut encoded = String::with_capacity(64);
    append_bytes(&mut encoded, &identity);
    encoded
}

fn append_bytes(encoded: &mut String, bytes: &[u8]) {
    const HEX: &[u8; 16] = b"0123456789abcdef";
    for byte in bytes {
        encoded.push(HEX[usize::from(byte >> 4)] as char);
        encoded.push(HEX[usize::from(byte & 0x0f)] as char);
    }
}
