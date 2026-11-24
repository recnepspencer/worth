use crate::merge::data::identity::IdentityBasisDeclaration;
use crate::merge::data::policy::AspectMergePolicyDeclaration;

use super::{AspectContractPlanRevision, DeclaredAspectContractBinding};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct KindAspectContractDeclarations {
    pub plan_revision: AspectContractPlanRevision,
    pub aspects: Vec<DeclaredAspectContractBinding>,
    pub identity_declarations: Vec<IdentityBasisDeclaration>,
    pub merge_policy_declarations: Vec<AspectMergePolicyDeclaration>,
}

impl KindAspectContractDeclarations {
    pub fn new(aspects: Vec<DeclaredAspectContractBinding>) -> Self {
        Self {
            plan_revision: AspectContractPlanRevision(0),
            aspects,
            identity_declarations: Vec::new(),
            merge_policy_declarations: Vec::new(),
        }
    }

    pub fn with_identity_declarations(
        mut self,
        identity_declarations: Vec<IdentityBasisDeclaration>,
    ) -> Self {
        self.identity_declarations = identity_declarations;
        self
    }

    pub fn with_merge_policy_declarations(
        mut self,
        merge_policy_declarations: Vec<AspectMergePolicyDeclaration>,
    ) -> Self {
        self.merge_policy_declarations = merge_policy_declarations;
        self
    }
}

impl Default for KindAspectContractDeclarations {
    fn default() -> Self {
        Self::new(Vec::new())
    }
}
