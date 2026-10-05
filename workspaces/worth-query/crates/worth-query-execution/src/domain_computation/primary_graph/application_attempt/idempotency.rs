use crate::domain_computation::authorization::WorthQueryOperationScopeBinding;

mod capability_workflow;
mod encoding;
mod host_commit;
mod mutation_binding;
mod recorded_intent;
#[cfg(test)]
mod recorded_intent_tests;
#[cfg(test)]
mod tests;
mod workflow_definition;
mod workflow_instance;
mod workflow_proposal_context;
mod workflow_transition;

pub use capability_workflow::WorthQueryCapabilityWorkflowIdempotency;
use encoding::{append_identity_slot, append_scope_slot, encode_identity};
pub(in crate::domain_computation::primary_graph) use recorded_intent::WorthQueryRecordedIntentMatch;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
struct WorthQueryIdempotencyEntityIdentity {
    partition: u32,
    local_slot: u64,
    generation: u32,
}

/// The admitted principal and scope under one installed package and schema.
/// It names no runtime and no installation generation, since both are counted
/// per process and neither survives a restore, so the same request matches its
/// durable intent in every runtime that installs the same package, at any
/// generation, before and after a restore or reopen.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
struct WorthQueryIdempotencyScopeIdentity {
    package_identity: [u8; 32],
    schema_identity: [u8; 32],
    principal: WorthQueryIdempotencyEntityIdentity,
    scope: WorthQueryIdempotencyEntityIdentity,
}

/// The idempotency key of one commit, bound to the intent it commits.
///
/// No caller writes either identity. They derive from canonical encoding through
/// the typed constructors: `for_mutation_identities` for a request to a mutation
/// binding, from the identities `ApplicationMutationIdentities::encode` derived
/// once, which also names the binding so two bindings that share an operation
/// and an input type never replay each other; `for_host_commit` for a commit
/// whose effect program the host built itself, scoped to the operation it
/// commits under, the only constructor a host names; and, for Publication's
/// capability workflow entries alone, `WorthQueryCapabilityWorkflowIdempotency`
/// behind the publication boundary.
///
/// An accepted source expectation can add its source with `bind_idempotency`, and
/// at commit and at resolution the runtime also binds the admitted operation's
/// definition, its scope, its preconditions, and its governed input into the
/// intent. The intent is durable, so none of these names the runtime that
/// admitted the request. Reusing a key with the same intent finds the earlier
/// commit; reusing it with a different intent is intent drift.
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
    mutation_binding_identity: Option<[u8; 32]>,
}

impl WorthQueryApplicationIdempotencyBinding {
    pub(crate) const fn new(key_identity: [u8; 32], intent_identity: [u8; 32]) -> Self {
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
            mutation_binding_identity: None,
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

    /// Every intent slot, without the encoding's version; `intent_text` is
    /// what a key records.
    fn intent_slots(self) -> String {
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
        mutation_binding::append_identity_slot(&mut encoded, self.mutation_binding_identity);
        encoded
    }

    pub(in crate::domain_computation::primary_graph) const fn bind_source(
        mut self,
        source_identity: Option<&[u8; 32]>,
    ) -> Self {
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
            mutation_binding_identity: self.mutation_binding_identity,
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
            mutation_binding_identity: self.mutation_binding_identity,
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
            mutation_binding_identity: self.mutation_binding_identity,
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
            mutation_binding_identity: self.mutation_binding_identity,
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
            mutation_binding_identity: self.mutation_binding_identity,
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
            mutation_binding_identity: self.mutation_binding_identity,
        }
    }
}
