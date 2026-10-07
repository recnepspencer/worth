use super::RecordPublicationDirector;
use crate::physical_runtime::{
    durability::{
        PhysicalMutationPreSealCancellationDenial, PhysicalMutationUnresolvedBindingObservation,
    },
    PhysicalPreSealCancellationDenial, PhysicalPreSealCancellationOutcome,
    PreparedPhysicalMutation,
};

impl RecordPublicationDirector {
    #[cfg_attr(not(feature = "certification-test-authority"), allow(dead_code))]
    pub(super) fn cancel_prepared_before_group_seal(
        &self,
        prepared: PreparedPhysicalMutation,
    ) -> PhysicalPreSealCancellationOutcome {
        self.settle_prepared_before_group_seal(
            prepared,
            crate::physical_runtime::PhysicalMutationProvenNoEffectCause::CancelledBeforeGroupSeal,
        )
    }

    pub(in crate::physical_runtime) fn settle_prepared_before_group_seal(
        &self,
        prepared: PreparedPhysicalMutation,
        terminal_cause: crate::physical_runtime::PhysicalMutationProvenNoEffectCause,
    ) -> PhysicalPreSealCancellationOutcome {
        if prepared.extent_copy_source().is_some() {
            return PhysicalPreSealCancellationOutcome::NotCancelled {
                prepared,
                cause: PhysicalPreSealCancellationDenial::CopyIntentRequiresResolution,
            };
        }
        let expected = PhysicalMutationUnresolvedBindingObservation::new(
            prepared.idempotency_identity(),
            prepared.request_fingerprint(),
            prepared.mutation_identity(),
        );
        match self
            .idempotency
            .cancel_before_group_seal(expected, terminal_cause)
        {
            Ok(terminal) => PhysicalPreSealCancellationOutcome::ProvenNoEffect(terminal),
            Err(denial) => PhysicalPreSealCancellationOutcome::NotCancelled {
                prepared,
                cause: match denial {
                    PhysicalMutationPreSealCancellationDenial::AuthorityReleased => {
                        PhysicalPreSealCancellationDenial::DurabilityAuthorityReleased
                    }
                    PhysicalMutationPreSealCancellationDenial::BindingMismatch => {
                        PhysicalPreSealCancellationDenial::BindingMismatch
                    }
                    PhysicalMutationPreSealCancellationDenial::GroupSealed => {
                        PhysicalPreSealCancellationDenial::GroupAlreadySealed
                    }
                    PhysicalMutationPreSealCancellationDenial::ReopenedUnresolved => {
                        PhysicalPreSealCancellationDenial::ReopenedUnresolved
                    }
                },
            },
        }
    }
}
