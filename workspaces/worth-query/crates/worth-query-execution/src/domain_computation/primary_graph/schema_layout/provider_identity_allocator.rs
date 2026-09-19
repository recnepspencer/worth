use worth_foundational::facade::AspectIdentity;
use worth_query_installation::facade::WorthQueryInstalledApplicationSchemaContractCatalog;

use super::{contract_space_exhausted, WorthQueryPrimaryGraphInstallationDenial};

pub(super) fn allocate_provider_aspect_identities(
    catalog: &WorthQueryInstalledApplicationSchemaContractCatalog,
) -> Result<[AspectIdentity; 4], WorthQueryPrimaryGraphInstallationDenial> {
    allocate_after(catalog.maximum_aspect_identity())
}

fn allocate_after(
    maximum_application_identity: Option<AspectIdentity>,
) -> Result<[AspectIdentity; 4], WorthQueryPrimaryGraphInstallationDenial> {
    let maximum = maximum_application_identity.map_or(0, |identity| identity.0);
    let first = maximum
        .checked_add(1)
        .ok_or_else(contract_space_exhausted)?;
    let second = maximum
        .checked_add(2)
        .ok_or_else(contract_space_exhausted)?;
    let third = maximum
        .checked_add(3)
        .ok_or_else(contract_space_exhausted)?;
    let fourth = maximum
        .checked_add(4)
        .ok_or_else(contract_space_exhausted)?;
    Ok([
        AspectIdentity(first),
        AspectIdentity(second),
        AspectIdentity(third),
        AspectIdentity(fourth),
    ])
}

#[cfg(test)]
mod tests {
    use worth_foundational::facade::AspectIdentity;

    use crate::domain_computation::primary_graph::WorthQueryPrimaryGraphInstallationDenialKind;

    use super::allocate_after;

    #[test]
    fn empty_application_catalog_reserves_provider_identities_one_through_four() {
        assert_eq!(
            allocate_after(None).unwrap(),
            [
                AspectIdentity(1),
                AspectIdentity(2),
                AspectIdentity(3),
                AspectIdentity(4),
            ]
        );
    }

    #[test]
    fn maximum_minus_four_is_the_last_successful_application_identity() {
        assert_eq!(
            allocate_after(Some(AspectIdentity(u64::MAX - 4))).unwrap(),
            [
                AspectIdentity(u64::MAX - 3),
                AspectIdentity(u64::MAX - 2),
                AspectIdentity(u64::MAX - 1),
                AspectIdentity(u64::MAX),
            ]
        );
    }

    #[test]
    fn final_four_identity_positions_deny_provider_allocation() {
        for maximum in [u64::MAX - 3, u64::MAX - 2, u64::MAX - 1, u64::MAX] {
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
