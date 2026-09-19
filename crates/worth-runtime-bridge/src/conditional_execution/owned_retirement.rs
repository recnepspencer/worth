use std::collections::BTreeMap;
use std::sync::{Arc, PoisonError};

use worth_signal::facade::branch::{AdmittedSignalBranchBasis, SignalBranchIdentity};

use super::signal_port::BridgeConditionalSignalPort;
use super::{
    BridgeConditionalDenial, BridgeConditionalDenialKind, BridgeInstalledConditionalLowering,
    BridgeOwnedSignalRuntime,
};

impl BridgeOwnedSignalRuntime {
    pub fn retire_owned_conditional(
        &mut self,
        lowering: &Arc<BridgeInstalledConditionalLowering>,
    ) -> Result<(), BridgeConditionalDenial> {
        self.require_owned_conditional_lowering(lowering)?;
        self.signal_services()?
            .conditional_port(lowering)?
            .retire_installed_contract(lowering.signal_contract())
            .map_err(signal_retirement_denial)?;
        self.release_owned_conditional_lowering(lowering);
        Ok(())
    }

    /// Retires every live Signal definition on one exact branch and releases
    /// the Bridge custody of all superseded generations on that branch.
    pub fn retire_owned_conditionals_on_signal_branch(
        &mut self,
        basis: &AdmittedSignalBranchBasis,
    ) -> Result<usize, BridgeConditionalDenial> {
        self.require_current_signal_basis(basis)?;
        let branch_identity = basis.observation().branch_id();
        let installed = self
            .conditional_lowerings
            .read()
            .unwrap_or_else(PoisonError::into_inner);
        let matching = installed
            .values()
            .filter(|lowering| lowering_belongs_to_branch(lowering, branch_identity))
            .cloned()
            .collect::<Vec<_>>();
        if matching.is_empty() {
            return Ok(0);
        }
        let fallback = installed
            .values()
            .filter(|lowering| !lowering_belongs_to_branch(lowering, branch_identity))
            .max_by_key(|lowering| lowering.signal_definition_generation())
            .cloned()
            .ok_or_else(missing_fallback_denial)?;
        let current_by_node = current_lowerings_by_node(&matching);
        let retirement_port = conditional_retirement_port(basis, &matching, &current_by_node)?;
        drop(installed);

        for lowering in matching
            .iter()
            .filter(|lowering| !is_current_lowering(lowering, &current_by_node))
        {
            self.release_owned_conditional_lowering(lowering);
        }
        for lowering in current_by_node.values() {
            retirement_port
                .retire_installed_contract(lowering.signal_contract())
                .map_err(signal_retirement_denial)?;
            self.release_owned_conditional_lowering(lowering);
        }
        self.signal_services()?
            .publish_conditional_port_from_lowering(&fallback);
        Ok(matching.len())
    }

    fn require_current_signal_basis(
        &self,
        basis: &AdmittedSignalBranchBasis,
    ) -> Result<(), BridgeConditionalDenial> {
        self.signal_services()?
            .owner_services()
            .basis_port()
            .compare_current_exact(basis)
            .map_err(|denial| {
                BridgeConditionalDenial::new(
                    BridgeConditionalDenialKind::StaleLowering,
                    format!("conditional branch retirement rejected its exact basis: {denial:?}"),
                )
            })
    }

    fn require_owned_conditional_lowering(
        &self,
        lowering: &Arc<BridgeInstalledConditionalLowering>,
    ) -> Result<(), BridgeConditionalDenial> {
        let installed = self
            .conditional_lowerings
            .read()
            .unwrap_or_else(PoisonError::into_inner);
        match installed.get(super::contract::lowering_key(lowering)) {
            Some(retained) if Arc::ptr_eq(retained, lowering) => Ok(()),
            Some(_) => Err(foreign_lowering_denial(
                "owned conditional retirement did not carry the retained lowering",
            )),
            None => Err(foreign_lowering_denial(
                "owned conditional retirement did not match this Bridge runtime",
            )),
        }
    }

    fn release_owned_conditional_lowering(
        &mut self,
        lowering: &Arc<BridgeInstalledConditionalLowering>,
    ) {
        lowering.lease.revoke_liveness();
        self.owned_conditional_targets
            .write()
            .unwrap_or_else(PoisonError::into_inner)
            .unregister(lowering);
        let removed = self
            .conditional_lowerings
            .write()
            .unwrap_or_else(PoisonError::into_inner)
            .remove(super::contract::lowering_key(lowering))
            .expect("validated owned lowering remains installed until local release");
        debug_assert!(Arc::ptr_eq(&removed, lowering));
    }
}

fn current_lowerings_by_node(
    matching: &[Arc<BridgeInstalledConditionalLowering>],
) -> BTreeMap<worth_signal::facade::NodeId, Arc<BridgeInstalledConditionalLowering>> {
    let mut current = BTreeMap::new();
    for lowering in matching {
        current
            .entry(lowering.signal_node())
            .and_modify(|incumbent: &mut Arc<BridgeInstalledConditionalLowering>| {
                if lowering.signal_definition_generation()
                    > incumbent.signal_definition_generation()
                {
                    *incumbent = Arc::clone(lowering);
                }
            })
            .or_insert_with(|| Arc::clone(lowering));
    }
    current
}

fn conditional_retirement_port(
    basis: &AdmittedSignalBranchBasis,
    matching: &[Arc<BridgeInstalledConditionalLowering>],
    current_by_node: &BTreeMap<
        worth_signal::facade::NodeId,
        Arc<BridgeInstalledConditionalLowering>,
    >,
) -> Result<BridgeConditionalSignalPort, BridgeConditionalDenial> {
    if let Some(port) = matching.iter().find_map(|lowering| {
        lowering
            .signal_port()
            .filter(|port| port.issuance_basis().descriptor() == basis.descriptor())
    }) {
        return Ok(port);
    }
    current_by_node
        .values()
        .find_map(|lowering| {
            lowering.signal_port().and_then(|port| {
                port.reconstitute_at_basis(basis)
                    .ok()
                    .map(|(service, _)| BridgeConditionalSignalPort::shared(Arc::new(service)))
            })
        })
        .ok_or_else(|| {
            BridgeConditionalDenial::new(
                BridgeConditionalDenialKind::SignalContractInstallation,
                "conditional branch retirement could not re-admit its current Signal service",
            )
        })
}

fn is_current_lowering(
    lowering: &Arc<BridgeInstalledConditionalLowering>,
    current_by_node: &BTreeMap<
        worth_signal::facade::NodeId,
        Arc<BridgeInstalledConditionalLowering>,
    >,
) -> bool {
    current_by_node
        .get(&lowering.signal_node())
        .is_some_and(|current| Arc::ptr_eq(current, lowering))
}

fn lowering_belongs_to_branch(
    lowering: &BridgeInstalledConditionalLowering,
    branch_identity: &SignalBranchIdentity,
) -> bool {
    lowering.registry_key.branch() == branch_identity
}

fn signal_retirement_denial(error: impl std::fmt::Debug) -> BridgeConditionalDenial {
    BridgeConditionalDenial::new(
        BridgeConditionalDenialKind::SignalContractInstallation,
        format!("Signal denied owned conditional retirement: {error:?}"),
    )
}

fn foreign_lowering_denial(detail: &str) -> BridgeConditionalDenial {
    BridgeConditionalDenial::new(BridgeConditionalDenialKind::ForeignSignalGraph, detail)
}

fn missing_fallback_denial() -> BridgeConditionalDenial {
    BridgeConditionalDenial::new(
        BridgeConditionalDenialKind::SignalContractInstallation,
        "conditional branch retirement has no surviving extension basis",
    )
}
