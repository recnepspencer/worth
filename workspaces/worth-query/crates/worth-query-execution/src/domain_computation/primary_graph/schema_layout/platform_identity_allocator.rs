use worth_foundational::facade::AspectIdentity;
use worth_query_installation::facade::WorthQueryInstalledApplicationSchemaContractCatalog;

use super::{contract_space_exhausted, WorthQueryPrimaryGraphInstallationDenial};

pub(super) fn allocate_platform_aspect_identities(
    catalog: &WorthQueryInstalledApplicationSchemaContractCatalog,
) -> Result<[AspectIdentity; 14], WorthQueryPrimaryGraphInstallationDenial> {
    allocate_after(catalog.maximum_aspect_identity())
}

fn allocate_after(
    maximum_application_identity: Option<AspectIdentity>,
) -> Result<[AspectIdentity; 14], WorthQueryPrimaryGraphInstallationDenial> {
    let maximum = maximum_application_identity.map_or(0, |identity| identity.0);
    let mut next = maximum;
    let mut identities = [AspectIdentity(0); 14];
    for identity in &mut identities {
        next = next.checked_add(1).ok_or_else(contract_space_exhausted)?;
        *identity = AspectIdentity(next);
    }
    Ok(identities)
}

#[cfg(test)]
mod tests {
    use worth_foundational::facade::AspectIdentity;

    use crate::domain_computation::primary_graph::WorthQueryPrimaryGraphInstallationDenialKind;

    use super::allocate_after;

    #[test]
    fn empty_application_catalog_reserves_every_platform_identity() {
        assert_eq!(
            allocate_after(None).unwrap(),
            [
                AspectIdentity(1),
                AspectIdentity(2),
                AspectIdentity(3),
                AspectIdentity(4),
                AspectIdentity(5),
                AspectIdentity(6),
                AspectIdentity(7),
                AspectIdentity(8),
                AspectIdentity(9),
                AspectIdentity(10),
                AspectIdentity(11),
                AspectIdentity(12),
                AspectIdentity(13),
                AspectIdentity(14),
            ]
        );
    }

    #[test]
    fn maximum_minus_fourteen_is_the_last_successful_application_identity() {
        assert_eq!(
            allocate_after(Some(AspectIdentity(u64::MAX - 14))).unwrap(),
            [
                AspectIdentity(u64::MAX - 13),
                AspectIdentity(u64::MAX - 12),
                AspectIdentity(u64::MAX - 11),
                AspectIdentity(u64::MAX - 10),
                AspectIdentity(u64::MAX - 9),
                AspectIdentity(u64::MAX - 8),
                AspectIdentity(u64::MAX - 7),
                AspectIdentity(u64::MAX - 6),
                AspectIdentity(u64::MAX - 5),
                AspectIdentity(u64::MAX - 4),
                AspectIdentity(u64::MAX - 3),
                AspectIdentity(u64::MAX - 2),
                AspectIdentity(u64::MAX - 1),
                AspectIdentity(u64::MAX),
            ]
        );
    }

    #[test]
    fn final_fourteen_identity_positions_deny_platform_allocation() {
        for maximum in (u64::MAX - 13)..=u64::MAX {
            let denial = allocate_after(Some(AspectIdentity(maximum))).unwrap_err();
            assert_eq!(
                denial.kind(),
                WorthQueryPrimaryGraphInstallationDenialKind::InvalidSchemaMember
            );
            assert_eq!(
                denial.subject(),
                "application schema exhausts Relational aspect-contract identity space"
            );
        }
    }
}
