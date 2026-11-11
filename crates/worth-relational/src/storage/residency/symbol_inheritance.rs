//! A successor's closure is covered by its exact predecessor plus changed values.
use super::*;
use crate::storage::overlay::PartitionState;
use std::sync::Arc;
#[derive(Clone)]
pub(super) struct SymbolAdmissionBasis {
    partition: PartitionState,
    inherited: Option<Arc<SymbolAdmissionInheritance>>,
    requirements: Option<Arc<required_symbols::CapturedSymbolRequirements>>,
    head: RelationalRegionSymbolAdmissionIdentity,
    owned: RelationalRegionSymbolAdmissionIdentity,
    full: RelationalRegionSymbolAdmissionIdentity,
    prepared: Option<Arc<RegionSymbolChanges>>,
    publication: Option<SymbolPublicationKey>,
}
#[derive(Debug, Clone)]
pub(super) struct SymbolPublicationCertificate {
    pub(super) publication: SymbolPublicationKey,
    pub(super) full_at_birth: bool,
}
#[derive(Debug)]
pub(super) struct SymbolAdmissionInheritance {
    head: RelationalRegionSymbolAdmissionIdentity,
    owned: RelationalRegionSymbolAdmissionIdentity,
    pub(super) changes: Arc<RegionSymbolChanges>,
    pub(super) certificate: Option<SymbolPublicationCertificate>,
    full: bool,
}
impl SymbolAdmissionBasis {
    pub(super) fn head(lease: &RelationalResidencyLease<Head>) -> Self {
        Self::from_lease(
            lease,
            lease.resident_region().history_presence.is_complete(),
        )
    }
    pub(super) fn full(lease: &RelationalResidencyLease<Full>) -> Self {
        Self::from_lease(lease, true)
    }
    fn from_lease<L>(lease: &RelationalResidencyLease<L>, complete: bool) -> Self {
        Self {
            partition: lease.resident_region().partition.clone(),
            inherited: lease.cell().symbol_inheritance.get().cloned(),
            requirements: lease.resident_region().symbol_requirements.get().cloned(),
            head: lease.head_symbol_admission_identity(),
            owned: if complete {
                RelationalRegionSymbolAdmissionIdentity::full(lease.cell())
            } else {
                RelationalRegionSymbolAdmissionIdentity::owned(lease.cell())
            },
            full: RelationalRegionSymbolAdmissionIdentity::full(lease.cell()),
            prepared: None,
            publication: None,
        }
    }
    pub(super) fn prepared(
        mut self,
        changes: Arc<RegionSymbolChanges>,
        publication: Option<SymbolPublicationKey>,
    ) -> Self {
        self.prepared = Some(changes);
        self.publication = publication;
        self
    }
    pub(super) fn install_head(self, lease: &RelationalResidencyLease<Head>) {
        self.install(lease, false);
    }
    pub(super) fn install_full(self, lease: &RelationalResidencyLease<Full>) {
        self.install(lease, true);
    }
    fn install<L>(self, lease: &RelationalResidencyLease<L>, full: bool) {
        let changes = self.prepared.unwrap_or_else(|| {
            RegionSymbolChanges::between(&self.partition, &lease.resident_region().partition)
        });
        if let Some(requirements) = self
            .requirements
            .as_ref()
            .filter(|_| self.publication.is_none())
        {
            let retained = if changes.owned.visits == 0 && !changes.prefixes_changed {
                Some(if full {
                    requirements.clone()
                } else {
                    Arc::new(requirements.with_omitted_history())
                })
            } else if !full && changes.head.visits == 0 {
                Some(Arc::new(requirements.head_only()))
            } else {
                None
            };
            if let Some(retained) = retained {
                let _ = lease.resident_region().symbol_requirements.set(retained);
            }
        }
        // All aliases resolve immediately to the original predicate owner;
        // metadata-only freezes create no predecessor chain or repeated bill.
        if self.publication.is_none() && changes.unchanged() {
            let _ = lease.cell().symbol_aliases.set(
                symbol_admission_identity::SymbolAdmissionAliases {
                    head: self.head.clone(),
                    owned: self.owned.clone(),
                    full: self.full,
                },
            );
        }
        let certificate = self
            .publication
            .map(|publication| SymbolPublicationCertificate {
                publication,
                full_at_birth: full,
            })
            .or_else(|| {
                changes
                    .unchanged()
                    .then(|| {
                        self.inherited
                            .as_ref()
                            .and_then(|previous| previous.certificate.clone())
                    })
                    .flatten()
                    .map(|mut certificate| {
                        certificate.full_at_birth &= full;
                        certificate
                    })
            });
        let _ = lease
            .cell()
            .symbol_inheritance
            .set(Arc::new(SymbolAdmissionInheritance {
                head: self.head,
                owned: self.owned,
                changes,
                certificate,
                full,
            }));
    }
}
impl SymbolAdmissionInheritance {
    pub(super) fn predecessor(
        &self,
        full: bool,
    ) -> Option<(
        RelationalRegionSymbolAdmissionIdentity,
        &required_symbols::CollectedSymbols,
    )> {
        if full {
            self.full
                .then(|| (self.owned.clone(), self.changes.owned.as_ref()))
        } else {
            Some((self.head.clone(), self.changes.head.as_ref()))
        }
    }
    pub(super) fn owned_predecessor(
        &self,
    ) -> (
        RelationalRegionSymbolAdmissionIdentity,
        &required_symbols::CollectedSymbols,
    ) {
        (self.owned.clone(), self.changes.owned.as_ref())
    }
}
impl RelationalRegionRetention {
    pub(super) fn inherit_symbols(&self, previous: &Self, changes: Arc<RegionSymbolChanges>) {
        let basis = if let Some(lease) = previous.admitted_full_lease() {
            Some(SymbolAdmissionBasis::full(&lease))
        } else {
            previous
                .accepted_head_lease()
                .map(|lease| SymbolAdmissionBasis::head(&lease))
        };
        if let Some(basis) = basis {
            let basis = basis.prepared(changes, None);
            if let Some(lease) = self.admitted_full_lease() {
                basis.install_full(&lease);
            } else if let Some(lease) = self.accepted_head_lease() {
                basis.install_head(&lease);
            }
        }
    }
    pub(crate) fn admit_own_symbols(
        &self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        symbols: &crate::runtime::RuntimeSymbolTable,
    ) -> Result<(), RelationalResidencyDemandFailure> {
        if let Some(lease) = self.admitted_full_lease() {
            lease.admit_symbols(execution, symbols)
        } else if let Some(lease) = self.accepted_head_lease() {
            lease.admit_owned_symbols(execution, symbols)
        } else {
            Ok(())
        }
    }
}
