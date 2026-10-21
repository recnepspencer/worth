//! Non-executable custody of exact checkpoint roots verified by native readers.
use std::sync::Arc;

use super::prerequisite_denials::capacity_denial;
use crate::domain_computation::primary_graph::output_lineage::RecordedSettlementIdentity;
use crate::domain_computation::primary_graph::WorthQueryOutputDemandDenial;

/// The reader and lineage retain the complete funded facts and output witness.
/// This claim pins their exact settlement address through prerequisite custody.
/// It has no source epoch, producer, refresh route, or Ready delivery authority.
pub(in crate::domain_computation::primary_graph::application_output_demand::registry) struct CheckpointPrerequisiteClaims
{
    identities: Vec<Arc<RecordedSettlementIdentity>>,
    // Field order destroys the backing before refunding its final-owner charge.
}

impl CheckpointPrerequisiteClaims {
    pub(super) fn prepare(maximum: usize) -> Result<Self, WorthQueryOutputDemandDenial> {
        let mut identities = Vec::new();
        identities
            .try_reserve_exact(maximum)
            .map_err(|_| capacity_denial())?;
        Ok(Self { identities })
    }

    pub(super) fn retain_verified(&mut self, identity: &Arc<RecordedSettlementIdentity>) {
        assert!(self.identities.len() < self.identities.capacity());
        self.identities.push(Arc::clone(identity));
    }
}
