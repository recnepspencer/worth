//! Query borrows actual recovery state before the Home's first Native Arc.
use worth_proof::{AuthorityMarker, RelationalReadmitted};
use worth_relational::facade::{
    durability::{
        FreshRelationalSourceAdmission, RecoveredRelationalRuntimeAuthority, RecoveryOutcome,
    },
    runtime::RelationalRuntime,
};

pub struct UnsharedReadmittedApplicationHome {
    pub(super) relational: worth_store_relational::UnsharedReadmittedRelationalSection,
    pub(super) context: crate::home_context::HomeContext,
}
impl AuthorityMarker for UnsharedReadmittedApplicationHome {}
impl UnsharedReadmittedApplicationHome {
    pub fn home_identity(&self) -> worth_foundational::facade::HomeIdentity {
        self.context.descriptor.identity
    }
    pub fn incarnation(&self) -> worth_foundational::facade::HomeIncarnation {
        self.context.descriptor.incarnation
    }
    pub fn fresh_query_finalization_parts(
        &mut self,
    ) -> Result<
        (&mut RelationalRuntime, &FreshRelationalSourceAdmission),
        worth_store_relational::RelationalSectionReadmissionDenial,
    > {
        self.relational.home_fresh_query_finalization_parts()
    }
    pub fn recovered_query_finalization_parts(
        &mut self,
    ) -> Result<
        (
            &mut RelationalRuntime,
            &RecoveredRelationalRuntimeAuthority,
            &RecoveryOutcome,
        ),
        worth_store_relational::RelationalSectionReadmissionDenial,
    > {
        self.relational.home_recovered_query_finalization_parts()
    }
    pub fn share(self) -> RelationalReadmitted<crate::ReadmittedApplicationHome> {
        RelationalReadmitted::from_owner(crate::ReadmittedApplicationHome {
            relational: self.relational.share(),
            context: self.context,
        })
    }
}
