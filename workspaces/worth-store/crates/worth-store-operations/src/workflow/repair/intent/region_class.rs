use super::super::integrity_classification::IntegrityRepairRegionClass;
use worth_store_offline_verifier::{OfflineAuthorityClass, OperationalTruthRegion};

pub(super) fn repair_class(
    region: &OperationalTruthRegion,
    authority: OfflineAuthorityClass,
) -> IntegrityRepairRegionClass {
    match region {
        OperationalTruthRegion::DegradedDerivedRegion(_)
        | OperationalTruthRegion::RebuildableRegion(_)
            if authority == OfflineAuthorityClass::Derived =>
        {
            IntegrityRepairRegionClass::DerivedRebuildable
        }
        OperationalTruthRegion::UnrecoverableAuthorityRegion(_) => {
            IntegrityRepairRegionClass::Unrecoverable
        }
        OperationalTruthRegion::IndeterminateTruthRegion(_) => {
            IntegrityRepairRegionClass::Indeterminate
        }
        OperationalTruthRegion::QuarantinedRegion(_) => {
            IntegrityRepairRegionClass::QuarantineRequired
        }
        _ if authority == OfflineAuthorityClass::ContentAuthority => {
            IntegrityRepairRegionClass::ContentTrustedSourceRequired
        }
        _ => IntegrityRepairRegionClass::AuthorityTrustedSourceRequired,
    }
}
