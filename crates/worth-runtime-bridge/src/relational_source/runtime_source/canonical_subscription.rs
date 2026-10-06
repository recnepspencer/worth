use std::sync::Arc;

use worth_relational::facade::mvcc::{
    CompanionBranchCell, CompanionPreflightBudget, CompanionPreflightStop,
    PendingCompanionRegistration, PreparedPublicationCompanionEffect,
    PublicationCompanionPreflight, PublicationCompanionRegistration,
    PublicationCompanionRegistrationPort, PublicationCompanionRegistrationStop,
    RelationalPublicationCompanion,
};
use worth_relational::facade::runtime::PositionedRelationalSnapshot;

use super::RuntimeBridgeRelationalSource;

/// Preparation of one full canonical-envelope subscription, without Signal
/// targets. Required registration blocks writers until activation completes.
#[derive(Debug)]
pub struct PendingRelationalBridgeCanonicalSubscription {
    pending: PendingCompanionRegistration,
    port: PublicationCompanionRegistrationPort,
    runtime_instance_id: u64,
}

/// Managed custody of the single canonical-envelope consumer installed at the
/// Relational publication boundary. The native registration governs disposal.
#[derive(Clone, Debug)]
pub struct RelationalBridgeCanonicalSubscription {
    registration: PublicationCompanionRegistration,
    port: PublicationCompanionRegistrationPort,
}

#[derive(Debug)]
struct CanonicalEnvelopeDelivery {
    runtime_instance_id: u64,
    consumer: Arc<dyn RelationalPublicationCompanion>,
}

impl RuntimeBridgeRelationalSource {
    /// Attach a full canonical-envelope consumer to every writer of this owner.
    ///
    /// Preparation receives the original sealed envelope and its bounded native
    /// preflight context. It never lowers through Signal correspondence targets.
    pub fn begin_canonical_envelope_subscription(
        &self,
    ) -> Result<PendingRelationalBridgeCanonicalSubscription, PublicationCompanionRegistrationStop>
    {
        let port = self
            .runtime
            .with_runtime(|runtime| runtime.publication_companion_port());
        let pending = port.begin_required_registration()?;
        Ok(PendingRelationalBridgeCanonicalSubscription {
            pending,
            port,
            runtime_instance_id: self.runtime.runtime_instance_id(),
        })
    }
}

impl PendingRelationalBridgeCanonicalSubscription {
    pub fn mint_branch_cell<T: Send + Sync + 'static>(
        &self,
        selected: &PositionedRelationalSnapshot,
        initial: Arc<T>,
    ) -> Result<CompanionBranchCell<T>, PublicationCompanionRegistrationStop> {
        self.pending.mint_branch_cell(selected, initial)
    }

    pub fn activate(
        self,
        consumer: Arc<dyn RelationalPublicationCompanion>,
        budget: CompanionPreflightBudget,
    ) -> Result<RelationalBridgeCanonicalSubscription, PublicationCompanionRegistrationStop> {
        let delivery = Arc::new(CanonicalEnvelopeDelivery {
            runtime_instance_id: self.runtime_instance_id,
            consumer,
        });
        let registration = self.pending.activate(delivery, budget)?;
        Ok(RelationalBridgeCanonicalSubscription {
            registration,
            port: self.port,
        })
    }
}

impl RelationalBridgeCanonicalSubscription {
    pub fn mint_branch_cell<T: Send + Sync + 'static>(
        &self,
        selected: &PositionedRelationalSnapshot,
        initial: Arc<T>,
    ) -> Result<CompanionBranchCell<T>, PublicationCompanionRegistrationStop> {
        self.registration.mint_branch_cell(selected, initial)
    }

    pub fn close(&self) -> Result<(), PublicationCompanionRegistrationStop> {
        self.port.remove_required(&self.registration)
    }
}

impl RelationalPublicationCompanion for CanonicalEnvelopeDelivery {
    fn prepare(
        &self,
        context: &mut PublicationCompanionPreflight<'_>,
    ) -> Result<PreparedPublicationCompanionEffect, CompanionPreflightStop> {
        context.claim_work(1)?;
        if context.runtime_instance_id() != self.runtime_instance_id {
            return Err(CompanionPreflightStop::ForeignCell);
        }
        // Keep only the stable owner binding here: retaining a runtime handle
        // in its participant would
        // create an ownership cycle and invite callback mutex re-entry.
        self.consumer.prepare(context)
    }
}
