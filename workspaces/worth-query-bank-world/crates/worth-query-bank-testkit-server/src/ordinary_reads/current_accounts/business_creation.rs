//! A real known business without an account, with lawful owner and teller.
use super::super::fixture::{
    block_on, runtime, BankBusinessOwnerSeed, BankEmployeeAssignmentSeed, BankPrincipalSeed,
    BankSnapshotBuilder, BankWorldSeed, CausalCredential, DynamicIdentity, TestIdentityWorld,
};
use super::*;
pub struct BusinessCreationFixture {
    pub world: TestIdentityWorld,
    pub institution: InstitutionId,
    pub business: BusinessId,
    owner: DynamicIdentity,
    teller: DynamicIdentity,
}
impl BusinessCreationFixture {
    fn authenticate(&self, identity: &DynamicIdentity) -> BankAuthenticatedPrincipal {
        block_on(self.world.runtime.authenticate_with(
            &self.world.authentication,
            CausalCredential::for_identity(identity),
            &request_scope(),
        ))
        .unwrap()
    }
    pub fn owner(&self) -> BankAuthenticatedPrincipal {
        self.authenticate(&self.owner)
    }
    pub fn teller(&self) -> BankAuthenticatedPrincipal {
        self.authenticate(&self.teller)
    }
}
pub fn business_creation_world() -> BusinessCreationFixture {
    let institution = InstitutionId::new(1).unwrap();
    let business = BusinessId::new(1).unwrap();
    let owner_id = BankPrincipalId::new(1).unwrap();
    let teller_id = BankPrincipalId::new(2).unwrap();
    let owner = DynamicIdentity::new("managed-business-owner");
    let teller = DynamicIdentity::new("managed-business-teller");
    let snapshot = BankSnapshotBuilder::new(BankSnapshotVersion::new(1).unwrap())
        .institution(institution)
        .business(business)
        .principal(owner_id)
        .principal(teller_id)
        .institution_cash_account(AccountId::new(100).unwrap(), institution)
        .build()
        .unwrap();
    let seed = BankWorldSeed::new(snapshot)
        .business_owner(BankBusinessOwnerSeed::new(business, owner_id))
        .employee(BankEmployeeAssignmentSeed::new(
            EmployeeAssignmentId::new(1).unwrap(),
            institution,
            teller_id,
            EmployeeRole::Teller,
        ))
        .principal(BankPrincipalSeed::enabled(owner_id, owner.external()))
        .principal(BankPrincipalSeed::enabled(teller_id, teller.external()));
    BusinessCreationFixture {
        world: runtime(seed),
        institution,
        business,
        owner,
        teller,
    }
}
