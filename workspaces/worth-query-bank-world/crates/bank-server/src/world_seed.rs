use bank_domain::estate::BankEstateWorld;
use bank_domain::model::{
    BankPrincipalId, BusinessId, EmployeeAssignmentId, EmployeeRole, InstitutionId,
};
use bank_domain::proposals::BankSnapshot;

use crate::{BankPrincipalSeed, BankProgramDeploymentGrantSeed};

pub struct BankBusinessOwnerSeed {
    business: BusinessId,
    principal: BankPrincipalId,
}

impl BankBusinessOwnerSeed {
    pub const fn new(business: BusinessId, principal: BankPrincipalId) -> Self {
        Self {
            business,
            principal,
        }
    }

    pub(crate) const fn business(&self) -> BusinessId {
        self.business
    }

    pub(crate) const fn principal(&self) -> BankPrincipalId {
        self.principal
    }
}

pub struct BankEmployeeAssignmentSeed {
    id: EmployeeAssignmentId,
    institution: InstitutionId,
    principal: BankPrincipalId,
    role: EmployeeRole,
}

impl BankEmployeeAssignmentSeed {
    pub const fn new(
        id: EmployeeAssignmentId,
        institution: InstitutionId,
        principal: BankPrincipalId,
        role: EmployeeRole,
    ) -> Self {
        Self {
            id,
            institution,
            principal,
            role,
        }
    }

    pub(crate) const fn id(&self) -> EmployeeAssignmentId {
        self.id
    }

    pub(crate) const fn institution(&self) -> InstitutionId {
        self.institution
    }

    pub(crate) const fn principal(&self) -> BankPrincipalId {
        self.principal
    }

    pub(crate) const fn role(&self) -> EmployeeRole {
        self.role
    }
}

pub struct BankWorldSeed {
    snapshot: BankSnapshot,
    principals: Vec<BankPrincipalSeed>,
    business_owners: Vec<BankBusinessOwnerSeed>,
    employees: Vec<BankEmployeeAssignmentSeed>,
    estate: Option<BankEstateWorld>,
    deployment_grants: Vec<BankProgramDeploymentGrantSeed>,
}

impl BankWorldSeed {
    pub fn new(snapshot: BankSnapshot) -> Self {
        Self {
            snapshot,
            principals: Vec::new(),
            business_owners: Vec::new(),
            employees: Vec::new(),
            estate: None,
            deployment_grants: Vec::new(),
        }
    }

    /// Issues only the explicitly supplied deployment permission at bootstrap.
    pub fn program_deployment_grant(mut self, grant: BankProgramDeploymentGrantSeed) -> Self {
        self.deployment_grants.push(grant);
        self
    }

    pub fn principal(mut self, principal: BankPrincipalSeed) -> Self {
        self.principals.push(principal);
        self
    }

    pub fn business_owner(mut self, owner: BankBusinessOwnerSeed) -> Self {
        self.business_owners.push(owner);
        self
    }

    pub fn employee(mut self, employee: BankEmployeeAssignmentSeed) -> Self {
        self.employees.push(employee);
        self
    }

    pub fn estate(mut self, estate: BankEstateWorld) -> Self {
        self.estate = Some(estate);
        self
    }

    pub(crate) fn into_parts(
        self,
    ) -> (
        BankSnapshot,
        Vec<BankPrincipalSeed>,
        Vec<BankBusinessOwnerSeed>,
        Vec<BankEmployeeAssignmentSeed>,
        Option<BankEstateWorld>,
        Vec<BankProgramDeploymentGrantSeed>,
    ) {
        (
            self.snapshot,
            self.principals,
            self.business_owners,
            self.employees,
            self.estate,
            self.deployment_grants,
        )
    }
}
