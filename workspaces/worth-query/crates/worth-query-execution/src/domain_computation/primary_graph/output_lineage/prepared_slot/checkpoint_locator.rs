//! Admission and retained ownership of the managed publication's portable locator.

use super::denial;
use super::PreparedOutputLineageSlot;
use crate::domain_computation::primary_graph::output_lineage::native_prior_checkpoint::NativePriorCheckpointLocator;
use crate::domain_computation::primary_graph::WorthQueryOutputDemandDenial;
use crate::domain_computation::primary_graph::WorthQueryOutputDemandDenialKind;

impl PreparedOutputLineageSlot {
    pub(in crate::domain_computation::primary_graph) fn retain_native_prior_checkpoint(
        &mut self,
        producer: &str,
        source: [u8; 32],
    ) -> Result<(), WorthQueryOutputDemandDenial> {
        use WorthQueryOutputDemandDenialKind as Kind;
        assert!(self.native_prior_checkpoint.is_none());
        let mut owned_producer = String::new();
        owned_producer
            .try_reserve_exact(producer.len())
            .map_err(|_| denial(Kind::RetentionBudgetExceeded))?;
        owned_producer.push_str(producer);
        self.native_prior_checkpoint = Some(NativePriorCheckpointLocator::Performed {
            producer: owned_producer,
            source,
        });
        Ok(())
    }
}
