//! Original occurrence, installed association and target incarnation form one retry intent.
mod canonical_identity;
use super::WorthQueryCorrectionCommandKey;

/// Immutable descriptive intent; this value grants no permission to publish.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct WorthQueryCorrectionIntent {
    identity: [u8; 32],
    command_key: WorthQueryCorrectionCommandKey,
    replay_definition_identity: Option<[u8; 32]>,
}

impl WorthQueryCorrectionIntent {
    pub fn derive_for_source(
        bindings: [&str; 3],
        occurrence: &worth_runtime_world::facade::CompositeCommitIdentity,
        branch: &worth_runtime_world::facade::ProductBranchObservation,
        original_intent: &[u8; 32],
        source_member: crate::domain_computation::application_aftermath::WorthQueryCorrectionSourceMember,
        command_key: &WorthQueryCorrectionCommandKey,
        lineage: &crate::domain_computation::application_aftermath::WorthQueryCorrectionLineageBinding,
    ) -> Self {
        use canonical_identity::{Coordinate, CorrectionIntentFields};
        let owner = occurrence.owner_identity();
        let lifecycle = branch.lifecycle_incarnation();
        let target_owner = lifecycle.owner_identity();
        let identity = canonical_identity::encode(CorrectionIntentFields {
            bindings,
            role: lineage.role(),
            source_owner: canonical_identity::scope(owner.durable(), owner.get()),
            source_commit: occurrence.ordinal(),
            source_member,
            target_owner: canonical_identity::scope(target_owner.durable(), target_owner.get()),
            target_branch: branch.branch_identity().name().as_str(),
            target_lifecycle: lifecycle.ordinal(),
            original_intent,
            lineage: [
                Some(lineage.root()),
                lineage.selected_alternative(),
                lineage.active_reversal(),
                lineage.prior_transition(),
            ]
            .map(|coordinate| coordinate.map(Coordinate::from_source)),
        });
        Self {
            identity,
            command_key: *command_key,
            replay_definition_identity: None,
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
