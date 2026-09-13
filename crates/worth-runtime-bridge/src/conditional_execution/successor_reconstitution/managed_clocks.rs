use super::BridgePreparedConditionalReconstitution;
use crate::conditional_execution::{
    BridgeManagedClockBinding, BridgeManagedClockInstallationParts, BridgeManagedTemporalDenial,
    BridgeManagedTemporalDenialKind, BridgeManagedTemporalIntentReconciliation,
    BridgeManagedTemporalIntentReconciliationParts,
};
use std::sync::{Arc, Mutex};

impl BridgePreparedConditionalReconstitution {
    pub fn install_managed_clock(
        &mut self,
        parts: BridgeManagedClockInstallationParts<'_>,
    ) -> Result<BridgeManagedClockBinding, BridgeManagedTemporalDenial> {
        let key = crate::conditional_execution::contract::lowering_key(parts.lowering);
        if !self
            .lowerings
            .get(&key)
            .is_some_and(|installed| Arc::ptr_eq(installed, parts.lowering))
        {
            return Err(denied(
                "managed clock requires its exact reconstructed lowering",
            ));
        }
        let declaration =
            crate::conditional_execution::managed_time::BridgeManagedClockDeclaration::admit(
                self.runtime_key,
                &self.retention,
                parts,
            )?;
        if self.managed_clocks.contains_key(declaration.identity()) {
            return Err(denied("duplicate reconstructed managed clock"));
        }
        let identity = Arc::clone(declaration.identity());
        let binding = declaration.binding();
        self.managed_clocks
            .insert(identity, Arc::new(Mutex::new(declaration.seal()?)));
        Ok(binding)
    }

    pub fn reconcile_managed_temporal_intent(
        &self,
        parts: BridgeManagedTemporalIntentReconciliationParts<'_>,
    ) -> Result<BridgeManagedTemporalIntentReconciliation, BridgeManagedTemporalDenial> {
        if parts.binding.bridge_runtime_key != self.runtime_key {
            return Err(denied("foreign reconstructed clock binding"));
        }
        let lane = self
            .managed_clocks
            .get(&parts.binding.binding_identity)
            .ok_or_else(|| denied("reconstructed clock is absent"))?;
        let mut lane = crate::conditional_execution::managed_time::lock_lane(lane)?;
        crate::conditional_execution::managed_time::reconcile_intent_in_lane(&mut lane, parts)
    }
}

fn denied(detail: &str) -> BridgeManagedTemporalDenial {
    BridgeManagedTemporalDenial::new(BridgeManagedTemporalDenialKind::ForeignClockBinding, detail)
}
