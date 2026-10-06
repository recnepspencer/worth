//! Admission and retained ownership of the managed publication's portable locator.

use super::{denial, PreparedOutputLineageSlot};
use crate::domain_computation::primary_graph::{
    output_lineage::{
        invalidation::InvalidationEditAdmission,
        native_prior_checkpoint::NativePriorCheckpointLocator,
    },
    WorthQueryOutputDemandDenial, WorthQueryOutputDemandDenialKind,
};
use worth_relational::facade::mvcc::CompanionPreflightStop;

impl PreparedOutputLineageSlot {
    pub(in crate::domain_computation::primary_graph) fn retain_native_prior_checkpoint(
        &mut self,
        producer: &str,
        source: [u8; 32],
        admission: &mut InvalidationEditAdmission,
    ) -> Result<(), WorthQueryOutputDemandDenial> {
        use WorthQueryOutputDemandDenialKind as Kind;
        assert!(self.native_prior_checkpoint.is_none());
        let bytes = producer.len() as u64;
        admission
            .charge_external_work(
                bytes
                    .checked_add(32)
                    .ok_or_else(|| denial(Kind::WorkBudgetExceeded))?,
            )
            .map_err(|_| denial(Kind::WorkBudgetExceeded))?;
        admission
            .admit_read_scratch(bytes)
            .map_err(scratch_denial)?;
        self.retained_capacity
            .as_mut()
            .expect("prepared lineage retains capacity before effects")
            .reserve_additional(bytes)?;
        let mut owned_producer = String::new();
        owned_producer
            .try_reserve_exact(producer.len())
            .map_err(|_| denial(Kind::RetentionBudgetExceeded))?;
        owned_producer.push_str(producer);
        self.native_prior_checkpoint = Some(NativePriorCheckpointLocator {
            producer: owned_producer,
            source,
        });
        Ok(())
    }
}

fn scratch_denial(stop: CompanionPreflightStop) -> WorthQueryOutputDemandDenial {
    use WorthQueryOutputDemandDenialKind as Kind;
    match stop {
        CompanionPreflightStop::WorkExhausted { .. }
        | CompanionPreflightStop::WorkCounterOverflow => denial(Kind::WorkBudgetExceeded),
        _ => denial(Kind::RetentionBudgetExceeded),
    }
}
