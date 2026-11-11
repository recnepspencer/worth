//! Original occurrence, installed association and target incarnation form one retry intent.
use super::*;
mod canonical_identity;
use worth_query_declaration::facade::{
    application_aftermath::ApplicationInverseBinding,
    application_operation::ApplicationMutationBinding,
};

/// Immutable descriptive intent; this value grants no permission to publish.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct WorthQueryCorrectionIntent {
    identity: [u8; 32],
    command_key: WorthQueryCorrectionCommandKey,
    replay_definition_identity: Option<[u8; 32]>,
}

impl WorthQueryCorrectionIntent {
    pub(super) const fn identity(&self) -> [u8; 32] {
        self.identity
    }
    pub(crate) fn causal_key(&self) -> String {
        use std::fmt::Write;
        let mut key = String::from("recorded-correction:");
        for byte in self.command_key.identity() {
            write!(&mut key, "{byte:02x}").expect("String formatting cannot refuse");
        }
        key
    }
    pub(super) fn from_disclosed_source<
        Schema: ApplicationSchema,
        C: ApplicationCorrectionBinding<Schema>,
    >(
        target: &WorthQuerySelectedProductOperation<'_, Schema>,
        source: &WorthQuerySelectedCorrectionSource<'_, Schema>,
        command_key: &WorthQueryCorrectionCommandKey,
        lineage: &crate::domain_computation::application_aftermath::WorthQueryCorrectionLineageBinding,
    ) -> Self {
        use crate::domain_computation::application_aftermath::WorthQueryAftermathCausalRole as Role;
        use canonical_identity::{Coordinate, CorrectionIntentFields};
        let occurrence = source.ancestry().source().identity();
        let branch = target.product().observation();
        let original_intent = source.occurrence().original_intent();
        let action = match lineage.role() {
            Role::RecordedReverse => ApplicationInverseBinding::<Schema, C>::IDENTITY,
            Role::RecordedReapply => C::Source::IDENTITY,
        };
        let owner = occurrence.owner_identity();
        let lifecycle = branch.lifecycle_incarnation();
        let target_owner = lifecycle.owner_identity();
        let identity = canonical_identity::encode(CorrectionIntentFields {
            bindings: [C::IDENTITY, C::Source::IDENTITY, action],
            role: lineage.role(),
            source_owner: canonical_identity::scope(owner.durable(), owner.get()),
            source_commit: occurrence.ordinal(),
            target_owner: canonical_identity::scope(target_owner.durable(), target_owner.get()),
            target_branch: branch.branch_identity().name().as_str(),
            target_lifecycle: lifecycle.ordinal(),
            original_intent: &original_intent,
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

    pub(super) fn with_recorded_definition_data(mut self, definition: Option<[u8; 32]>) -> Self {
        self.replay_definition_identity = definition;
        self
    }

    pub(super) fn with_recorded_definition(
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

    pub(super) fn matches(
        &self,
        binding: &crate::domain_computation::primary_graph::WorthQueryApplicationIdempotencyBinding,
    ) -> bool {
        binding.correction_identity() == Some(&self.identity)
    }
}
