//! Stable installed declaration identity survives revocation of installation authority.
use super::{
    BridgeConditionalContract, BridgeConditionalLocation, BridgeInstalledConditionalLowering,
};
use std::{cmp::Ordering, sync::Arc};

#[derive(Clone)]
pub struct BridgeAuthoredConditionalDeclaration {
    pub(super) installation: worth_proof::BridgeProgramInstallationBinding,
    pub(super) _installation: Arc<BridgeInstalledConditionalLowering>,
    pub(super) key: DeclarationKey,
}
#[derive(Clone)]
pub(super) struct DeclarationKey {
    contract: Arc<BridgeConditionalContract>,
    location: Arc<BridgeConditionalLocation>,
}
impl DeclarationKey {
    pub(super) fn of(lowering: &BridgeInstalledConditionalLowering) -> Self {
        Self {
            contract: Arc::clone(&lowering.contract),
            location: Arc::clone(&lowering.location),
        }
    }
}
impl PartialEq for DeclarationKey {
    fn eq(&self, other: &Self) -> bool {
        self.cmp(other) == Ordering::Equal
    }
}
impl Eq for DeclarationKey {}
impl PartialOrd for DeclarationKey {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}
impl Ord for DeclarationKey {
    fn cmp(&self, other: &Self) -> Ordering {
        self.location
            .cmp(&other.location)
            .then_with(|| self.contract.identity().cmp(other.contract.identity()))
    }
}
pub(super) type DeclarationIndex =
    std::collections::BTreeMap<DeclarationKey, Arc<BridgeInstalledConditionalLowering>>;
