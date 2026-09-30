use worth_foundational::facade::AspectIdentity;
use worth_query_installation::facade::WorthQueryInstalledApplicationSchemaContractCatalog;

use super::{contract_space_exhausted, WorthQueryPrimaryGraphInstallationDenial};

pub(super) fn allocate_platform_aspect_identities(
    catalog: &WorthQueryInstalledApplicationSchemaContractCatalog,
) -> Result<[AspectIdentity; 16], WorthQueryPrimaryGraphInstallationDenial> {
    allocate_after(catalog.maximum_aspect_identity())
}

fn allocate_after(
    maximum_application_identity: Option<AspectIdentity>,
) -> Result<[AspectIdentity; 16], WorthQueryPrimaryGraphInstallationDenial> {
    let maximum = maximum_application_identity.map_or(0, |identity| identity.0);
    let mut next = maximum;
    let mut identities = [AspectIdentity(0); 16];
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
            std::array::from_fn(|index| AspectIdentity(index as u64 + 1))
        );
    }

    #[test]
    fn maximum_minus_sixteen_is_the_last_successful_application_identity() {
        assert_eq!(
            allocate_after(Some(AspectIdentity(u64::MAX - 16))).unwrap(),
            std::array::from_fn(|index| AspectIdentity(u64::MAX - 15 + index as u64))
        );
    }

    #[test]
    fn final_sixteen_identity_positions_deny_platform_allocation() {
        for maximum in (u64::MAX - 15)..=u64::MAX {
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
