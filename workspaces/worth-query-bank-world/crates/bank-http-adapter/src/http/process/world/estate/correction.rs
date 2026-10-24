//! Explicit grants for the process world's account correction and authority change.
use super::*;

#[derive(Deserialize)]
pub struct BankHttpProcessEstateCorrectionWorld {
    pub freeze_grant: u64,
    pub revoke_grant: u64,
}

impl BankHttpProcessEstateCorrectionWorld {
    pub(super) fn install(
        self,
        mut world: BankEstateWorld,
        institution: InstitutionId,
        branch: BranchId,
        estate: EstateCaseId,
        account: AccountId,
        specialist: BankPrincipalId,
    ) -> Result<BankEstateWorld, ()> {
        for (id, operation, account) in [
            (
                self.freeze_grant,
                EstateCapabilityOperation::FreezeAccount,
                Some(account),
            ),
            (
                self.revoke_grant,
                EstateCapabilityOperation::RevokeCapability,
                None,
            ),
        ] {
            world = world.with_grant(EstateCapabilityGrant {
                id: CapabilityGrantId::new(id).ok_or(())?,
                grantor: specialist,
                grantee: specialist,
                scope: capability_scope(
                    institution,
                    branch,
                    estate,
                    operation,
                    EstateCapabilityPurpose::EstateAdministration,
                    account,
                    None,
                )?,
                parent: None,
                status: CapabilityGrantStatus::Active,
            });
        }
        Ok(world)
    }
}
