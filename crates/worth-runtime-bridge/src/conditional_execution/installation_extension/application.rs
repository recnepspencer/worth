use std::sync::Arc;

use super::super::{
    BridgeConditionalDenial, BridgeConditionalDenialKind, BridgeInstalledConditionalLowering,
    BridgeOwnedSignalRuntime,
};

pub struct BridgeAppliedConditionalInstallationExtension {
    pub(super) signal: Option<
        worth_signal::facade::branch::SignalConditionalInstallationExtensionCompletion<(), (), ()>,
    >,
    pub(super) activation: super::super::lowering_installation::BridgePreparedConditionalActivation,
}

impl BridgeOwnedSignalRuntime {
    pub(crate) fn apply_owned_conditional_installation_extension<E, Ctx>(
        &self,
        transaction: &mut worth_signal::facade::SignalTransaction<'_, (), (), E, Ctx, ()>,
        prepared: super::BridgePreparedConditionalInstallationExtension,
    ) -> Result<BridgeAppliedConditionalInstallationExtension, BridgeConditionalDenial> {
        let signal = prepared
            .signal_port
            .apply_installation_extension(transaction, prepared.signal)
            .map_err(|denial| {
                BridgeConditionalDenial::new(
                    BridgeConditionalDenialKind::SignalContractInstallation,
                    format!("Signal installation extension was denied: {denial:?}"),
                )
            })?;
        self.admit_conditional_activation_contract(&prepared.activation, signal.contract())?;
        Ok(BridgeAppliedConditionalInstallationExtension {
            signal: Some(signal),
            activation: prepared.activation,
        })
    }

    pub(crate) fn complete_owned_conditional_installation_activation(
        &self,
        applied: &mut BridgeAppliedConditionalInstallationExtension,
        binding: worth_signal::facade::branch::SignalConditionalDefinitionAdvanceBinding,
    ) -> Result<(), BridgeConditionalDenial> {
        if let Some(signal) = applied.signal.take() {
            let (signal_contract, definition_custody, signal_port) =
                signal.into_retained_parts(binding).map_err(|denial| {
                    BridgeConditionalDenial::new(
                        BridgeConditionalDenialKind::SignalContractInstallation,
                        format!("Signal successor activation was denied: {denial:?}"),
                    )
                })?;
            applied
                .activation
                .lowering
                .initialize_signal_contract(signal_contract);
            applied
                .activation
                .lowering
                .initialize_definition_custody(definition_custody);
            applied
                .activation
                .lowering
                .initialize_reserved_signal_port(signal_port);
        }
        Ok(())
    }

    pub(crate) fn commit_owned_conditional_installation_extension(
        &self,
        applied: BridgeAppliedConditionalInstallationExtension,
    ) -> Arc<BridgeInstalledConditionalLowering> {
        self.commit_admitted_conditional_activation(applied.activation)
    }
}
