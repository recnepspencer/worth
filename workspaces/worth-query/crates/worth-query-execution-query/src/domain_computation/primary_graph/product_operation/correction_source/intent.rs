//! Original occurrence, installed association and target incarnation form one retry intent.
use super::WorthQueryCorrectionCommandKey;

/// Immutable descriptive intent; this value grants no permission to publish.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct WorthQueryCorrectionIntent {
    identity: [u8; 32],
    command_key: WorthQueryCorrectionCommandKey,
    replay_definition_identity: Option<[u8; 32]>,
}

impl WorthQueryCorrectionIntent {
    pub fn from_parts(
        _seal: crate::ports::WorthQueryQueryCrateSeal,
        identity: [u8; 32],
        command_key: WorthQueryCorrectionCommandKey,
        replay_definition_identity: Option<[u8; 32]>,
    ) -> Self {
        Self {
            identity,
            command_key,
            replay_definition_identity,
        }
    }

    pub const fn identity(&self) -> [u8; 32] {
        self.identity
    }

    pub fn causal_key(&self) -> String {
        use std::fmt::Write;
        let mut key = String::from("recorded-correction:");
        for byte in self.command_key.identity() {
            write!(&mut key, "{byte:02x}").expect("String formatting cannot refuse");
        }
        key
    }

    pub fn with_recorded_definition_data(mut self, definition: Option<[u8; 32]>) -> Self {
        self.replay_definition_identity = definition;
        self
    }

    pub fn with_recorded_definition(
        mut self,
        recorded: &crate::domain_computation::primary_graph::WorthQueryApplicationIdempotencyBinding,
    ) -> Self {
        self.replay_definition_identity = recorded.conditional_definition_identity();
        self
    }

    pub fn bind_idempotency(
        self,
        binding: crate::domain_computation::primary_graph::WorthQueryApplicationIdempotencyBinding,
    ) -> crate::domain_computation::primary_graph::WorthQueryApplicationIdempotencyBinding {
        binding
            .bind_correction_command(&self.command_key.identity(), &self.identity)
            .bind_conditional_definition(self.replay_definition_identity.as_ref())
    }

    pub fn matches(
        &self,
        binding: &crate::domain_computation::primary_graph::WorthQueryApplicationIdempotencyBinding,
    ) -> bool {
        binding.correction_identity() == Some(&self.identity)
    }
}
