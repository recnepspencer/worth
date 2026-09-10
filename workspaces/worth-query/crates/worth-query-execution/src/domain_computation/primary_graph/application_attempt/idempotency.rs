use crate::domain_computation::authorization::WorthQueryOperationScopeBinding;

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
    operation_identity: Option<[u8; 32]>,
    operation_scope_identity: Option<WorthQueryIdempotencyScopeIdentity>,
    precondition_identity: Option<[u8; 32]>,
    governed_input_identity: Option<[u8; 32]>,
    governed_proposal_identity: Option<[u8; 32]>,
    conditional_definition_identity: Option<[u8; 32]>,
}

impl WorthQueryApplicationIdempotencyBinding {
    pub const fn new(key_identity: [u8; 32], intent_identity: [u8; 32]) -> Self {
        Self {
            key_identity,
            intent_identity,
            operation_identity: None,
            operation_scope_identity: None,
            precondition_identity: None,
            governed_input_identity: None,
            governed_proposal_identity: None,
            conditional_definition_identity: None,
        }
    }

    pub const fn key_identity(&self) -> &[u8; 32] {
        &self.key_identity
    }

    pub const fn intent_identity(&self) -> &[u8; 32] {
        &self.intent_identity
    }

    pub(in crate::domain_computation::primary_graph) fn key_text(self) -> String {
        encode_identity(self.key_identity)
    }

    pub(in crate::domain_computation::primary_graph) fn intent_text(self) -> String {
        let mut encoded = encode_identity(self.intent_identity);
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
        encoded
    }

    pub(in crate::domain_computation::primary_graph) const fn bind_operation(
        self,
        operation_identity: &[u8; 32],
    ) -> Self {
        Self {
            key_identity: self.key_identity,
            intent_identity: self.intent_identity,
            operation_identity: Some(*operation_identity),
            operation_scope_identity: self.operation_scope_identity,
            precondition_identity: self.precondition_identity,
            governed_input_identity: self.governed_input_identity,
            governed_proposal_identity: self.governed_proposal_identity,
            conditional_definition_identity: self.conditional_definition_identity,
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
        }
    }

    pub(in crate::domain_computation::primary_graph) const fn bind_preconditions(
        self,
        precondition_identity: Option<&[u8; 32]>,
    ) -> Self {
        Self {
            key_identity: self.key_identity,
            intent_identity: self.intent_identity,
            operation_identity: self.operation_identity,
            operation_scope_identity: self.operation_scope_identity,
            precondition_identity: match precondition_identity {
                Some(identity) => Some(*identity),
                None => None,
            },
            governed_input_identity: self.governed_input_identity,
            governed_proposal_identity: self.governed_proposal_identity,
            conditional_definition_identity: self.conditional_definition_identity,
        }
    }

    pub(in crate::domain_computation::primary_graph) const fn bind_governed_input(
        self,
        governed_input_identity: Option<&[u8; 32]>,
    ) -> Self {
        Self {
            key_identity: self.key_identity,
            intent_identity: self.intent_identity,
            operation_identity: self.operation_identity,
            operation_scope_identity: self.operation_scope_identity,
            precondition_identity: self.precondition_identity,
            governed_input_identity: match governed_input_identity {
                Some(identity) => Some(*identity),
                None => None,
            },
            governed_proposal_identity: self.governed_proposal_identity,
            conditional_definition_identity: self.conditional_definition_identity,
        }
    }

    pub(in crate::domain_computation::primary_graph) const fn bind_governed_proposal(
        self,
        governed_proposal_identity: Option<&[u8; 32]>,
    ) -> Self {
        Self {
            key_identity: self.key_identity,
            intent_identity: self.intent_identity,
            operation_identity: self.operation_identity,
            operation_scope_identity: self.operation_scope_identity,
            precondition_identity: self.precondition_identity,
            governed_input_identity: self.governed_input_identity,
            governed_proposal_identity: match governed_proposal_identity {
                Some(identity) => Some(*identity),
                None => None,
            },
            conditional_definition_identity: self.conditional_definition_identity,
        }
    }

    pub(in crate::domain_computation::primary_graph) const fn bind_conditional_definition(
        self,
        identity: Option<&[u8; 32]>,
    ) -> Self {
        Self {
            key_identity: self.key_identity,
            intent_identity: self.intent_identity,
            operation_identity: self.operation_identity,
            operation_scope_identity: self.operation_scope_identity,
            precondition_identity: self.precondition_identity,
            governed_input_identity: self.governed_input_identity,
            governed_proposal_identity: self.governed_proposal_identity,
            conditional_definition_identity: match identity {
                Some(identity) => Some(*identity),
                None => None,
            },
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

#[cfg(test)]
mod tests {
    use super::{
        WorthQueryApplicationIdempotencyBinding, WorthQueryIdempotencyEntityIdentity,
        WorthQueryIdempotencyScopeIdentity,
    };

    #[test]
    fn governed_proposal_is_a_private_part_of_idempotency_intent() {
        let baseline = WorthQueryApplicationIdempotencyBinding::new([1; 32], [2; 32]);
        let first = baseline.bind_governed_proposal(Some(&[3; 32]));
        let retry = baseline.bind_governed_proposal(Some(&[3; 32]));
        let drift = baseline.bind_governed_proposal(Some(&[4; 32]));

        assert_eq!(first.key_text(), retry.key_text());
        assert_eq!(first.intent_text(), retry.intent_text());
        assert_eq!(first.key_text(), drift.key_text());
        assert_ne!(first.intent_text(), drift.intent_text());
    }

    #[test]
    fn governed_input_is_a_private_part_of_idempotency_intent() {
        let baseline = WorthQueryApplicationIdempotencyBinding::new([1; 32], [2; 32]);
        let first = baseline.bind_governed_input(Some(&[3; 32]));
        let retry = baseline.bind_governed_input(Some(&[3; 32]));
        let drift = baseline.bind_governed_input(Some(&[4; 32]));

        assert_eq!(first.key_text(), retry.key_text());
        assert_eq!(first.intent_text(), retry.intent_text());
        assert_eq!(first.key_text(), drift.key_text());
        assert_ne!(first.intent_text(), drift.intent_text());
    }

    #[test]
    fn private_identity_slots_cannot_alias_each_other() {
        let baseline = WorthQueryApplicationIdempotencyBinding::new([1; 32], [2; 32]);
        let identity = [3; 32];
        let precondition = baseline.bind_preconditions(Some(&identity)).intent_text();
        let input = baseline.bind_governed_input(Some(&identity)).intent_text();
        let proposal = baseline
            .bind_governed_proposal(Some(&identity))
            .intent_text();

        assert_ne!(precondition, input);
        assert_ne!(precondition, proposal);
        assert_ne!(input, proposal);
    }

    #[test]
    fn every_private_identity_survives_combined_composition() {
        let baseline = WorthQueryApplicationIdempotencyBinding::new([1; 32], [2; 32]);
        let combined = baseline
            .bind_operation(&[3; 32])
            .bind_preconditions(Some(&[4; 32]))
            .bind_governed_input(Some(&[5; 32]))
            .bind_governed_proposal(Some(&[6; 32]));

        for drift in [
            baseline
                .bind_operation(&[7; 32])
                .bind_preconditions(Some(&[4; 32]))
                .bind_governed_input(Some(&[5; 32]))
                .bind_governed_proposal(Some(&[6; 32])),
            combined.bind_preconditions(Some(&[7; 32])),
            combined.bind_governed_input(Some(&[7; 32])),
            combined.bind_governed_proposal(Some(&[7; 32])),
        ] {
            assert_ne!(combined.intent_text(), drift.intent_text());
        }
    }

    #[test]
    fn admitted_principal_and_scope_are_distinct_idempotency_components() {
        let baseline = WorthQueryApplicationIdempotencyBinding::new([1; 32], [2; 32]);
        let mut first = baseline;
        first.operation_scope_identity = Some(scope_identity(10, 20));
        let mut principal_drift = baseline;
        principal_drift.operation_scope_identity = Some(scope_identity(11, 20));
        let mut scope_drift = baseline;
        scope_drift.operation_scope_identity = Some(scope_identity(10, 21));

        assert_ne!(first.intent_text(), principal_drift.intent_text());
        assert_ne!(first.intent_text(), scope_drift.intent_text());
    }

    fn scope_identity(principal_slot: u64, scope_slot: u64) -> WorthQueryIdempotencyScopeIdentity {
        WorthQueryIdempotencyScopeIdentity {
            runtime_authority: 3,
            binding_runtime: 4,
            binding_generation: 5,
            package_identity: [6; 32],
            schema_identity: [7; 32],
            principal: WorthQueryIdempotencyEntityIdentity {
                partition: 8,
                local_slot: principal_slot,
                generation: 9,
            },
            scope: WorthQueryIdempotencyEntityIdentity {
                partition: 8,
                local_slot: scope_slot,
                generation: 9,
            },
        }
    }
}
