//! Original occurrence, installed association and target incarnation form one retry intent.
use super::*;
use sha2::{Digest, Sha256};
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
        let mut digest = Sha256::new();
        let occurrence = source.ancestry().source().identity();
        let branch = target.product().observation();
        let original = source.occurrence().commit_evidence().original_authority();
        let action = match lineage.role() {
            crate::domain_computation::application_aftermath::WorthQueryAftermathCausalRole::RecordedReverse => ApplicationInverseBinding::<Schema, C>::IDENTITY,
            _ => C::Source::IDENTITY,
        };
        for part in [
            b"worth-query.recorded-correction.intent.v2".as_slice(),
            C::IDENTITY.as_bytes(),
            C::Source::IDENTITY.as_bytes(),
            action.as_bytes(),
            &lineage.role().code().to_be_bytes(),
            &occurrence.owner_identity().get().to_be_bytes(),
            &occurrence.ordinal().to_be_bytes(),
            branch.branch_identity().name().as_str().as_bytes(),
            &branch
                .lifecycle_incarnation()
                .owner_identity()
                .get()
                .to_be_bytes(),
            &branch.lifecycle_incarnation().ordinal().to_be_bytes(),
            original.idempotency_binding().intent_identity(),
        ] {
            digest.update((part.len() as u64).to_be_bytes());
            digest.update(part);
        }
        for coordinate in [
            Some(lineage.root()),
            lineage.selected_alternative(),
            lineage.active_reversal(),
            lineage.prior_transition(),
        ] {
            digest.update([u8::from(coordinate.is_some())]);
            if let Some(coordinate) = coordinate {
                for part in [
                    coordinate.world().owner_identity().get().to_be_bytes(),
                    coordinate.world().ordinal().to_be_bytes(),
                    coordinate.relational().commit_id.0.to_be_bytes(),
                    coordinate.relational().version_id.0.to_be_bytes(),
                ] {
                    digest.update(part);
                }
                let branch = coordinate.relational().branch_id.0.as_bytes();
                digest.update((branch.len() as u64).to_be_bytes());
                digest.update(branch);
            }
        }
        Self {
            identity: digest.finalize().into(),
            command_key: *command_key,
            replay_definition_identity: None,
        }
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
