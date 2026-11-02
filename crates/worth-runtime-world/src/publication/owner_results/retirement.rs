//! The results owner transfers its one Native basis without rewriting evidence.
use super::*;
impl CompositeOwnerExecutionResults {
    pub(crate) fn preflight_home_retirement(&self) -> Result<(), ()> {
        match &self.relational.result {
            CompositeRelationalOwnerResultKind::RetainedExact
            | CompositeRelationalOwnerResultKind::Published {
                settlement: Some(_),
                ..
            }
            | CompositeRelationalOwnerResultKind::Forked { .. } => Ok(()),
            CompositeRelationalOwnerResultKind::Published {
                settlement: None, ..
            }
            | CompositeRelationalOwnerResultKind::SettlementRequired { .. }
            | CompositeRelationalOwnerResultKind::DurabilityUnresolved { .. } => Err(()),
        }
    }
    pub(crate) fn take_native_for_retirement(&mut self) -> Option<AdmittedRelationalBranchBasis> {
        match &mut self.relational.result {
            CompositeRelationalOwnerResultKind::RetainedExact
            | CompositeRelationalOwnerResultKind::DurabilityUnresolved { .. } => None,
            CompositeRelationalOwnerResultKind::Published {
                successor_basis, ..
            }
            | CompositeRelationalOwnerResultKind::Forked {
                successor_basis, ..
            }
            | CompositeRelationalOwnerResultKind::SettlementRequired {
                successor_basis, ..
            } => successor_basis.take(),
        }
    }
}
