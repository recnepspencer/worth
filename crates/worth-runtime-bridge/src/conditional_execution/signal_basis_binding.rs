use std::sync::Arc;

use super::{
    BridgeConditionalDenial, BridgeConditionalDenialKind, BridgeInstalledConditionalLowering,
    BridgeOwnedSignalRuntime,
};

/// Exact pairing of one installed definition generation with one admitted
/// Signal branch basis. Product correspondence is established by the runtime
/// that owns the complete product observation.
pub struct BridgeConditionalSignalBasisBinding {
    pub(super) lowering: Arc<BridgeInstalledConditionalLowering>,
    pub(super) signal_port: super::signal_port::BridgeConditionalSignalPort,
}

impl BridgeOwnedSignalRuntime {
    pub(super) fn admit_exact_conditional_signal_basis(
        &self,
        anchor: &Arc<BridgeInstalledConditionalLowering>,
        basis: &worth_signal::facade::branch::AdmittedSignalBranchBasis,
    ) -> Result<BridgeConditionalSignalBasisBinding, BridgeConditionalDenial> {
        self.require_live_installed_lowering(anchor)?;
        let lowering = self
            .conditional_lowerings
            .read()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .exact_basis(basis, anchor.signal_node())
            .cloned()
            .ok_or_else(|| {
                BridgeConditionalDenial::new(
                    BridgeConditionalDenialKind::StaleLowering,
                    "selected Signal basis has no installed definition for this operation",
                )
            })?;
        if lowering.contract().identity() != anchor.contract().identity()
            || lowering.location() != anchor.location()
        {
            return Err(BridgeConditionalDenial::new(
                BridgeConditionalDenialKind::StaleLowering,
                "selected Signal basis resolved a different conditional operation",
            ));
        }
        self.admit_conditional_signal_basis(&lowering, basis)
    }

    pub(super) fn admit_conditional_signal_basis(
        &self,
        lowering: &Arc<BridgeInstalledConditionalLowering>,
        basis: &worth_signal::facade::branch::AdmittedSignalBranchBasis,
    ) -> Result<BridgeConditionalSignalBasisBinding, BridgeConditionalDenial> {
        self.require_live_installed_lowering(lowering)?;
        let installed_port = self.signal_services()?.conditional_port(lowering)?;
        let signal_port =
            if installed_port.issuance_basis().admission_identity() == basis.admission_identity() {
                installed_port.clone()
            } else {
                let reissued_port =
                    installed_port
                        .reissue_for_successor_basis(basis)
                        .map_err(|denial| {
                            BridgeConditionalDenial::new(
                                BridgeConditionalDenialKind::StaleLowering,
                                format!(
                                    "Signal definition generation admission was denied: {denial:?}"
                                ),
                            )
                        })?;
                reissued_port
                    .validate_installed_contract(lowering.signal_contract())
                    .map_err(|denial| {
                        BridgeConditionalDenial::new(
                            BridgeConditionalDenialKind::StaleLowering,
                            format!(
                                "selected Signal basis rejected the lowering generation: {denial:?}"
                            ),
                        )
                    })?;
                super::signal_port::BridgeConditionalSignalPort::shared(Arc::new(reissued_port))
            };
        Ok(BridgeConditionalSignalBasisBinding {
            lowering: Arc::clone(lowering),
            signal_port,
        })
    }
}

impl BridgeConditionalSignalBasisBinding {
    pub fn installed_lowering(&self) -> Arc<BridgeInstalledConditionalLowering> {
        Arc::clone(&self.lowering)
    }

    pub fn installed_lowering_ref(&self) -> &Arc<BridgeInstalledConditionalLowering> {
        &self.lowering
    }

    pub fn admission_identity(
        &self,
    ) -> &worth_signal::facade::branch::SignalBranchBasisAdmissionIdentity {
        self.signal_port.issuance_basis().admission_identity()
    }
}
