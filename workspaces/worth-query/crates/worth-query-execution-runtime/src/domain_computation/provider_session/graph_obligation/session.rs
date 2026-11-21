use crate::domain_computation::primary_graph::WorthQueryApplicationBasisIdentity;
use worth_query_admission::facade::graph_obligation::WorthQueryAdmittedGraphWorkPlan;
use worth_query_installation::facade::{
    ApplicationSchemaBindingIdentity, WorthQueryInstalledGraphObligationSetIdentity,
};
use worth_relational::facade::identity::EntityId;

use super::{
    WorthQueryGraphReadOwnerPort, WorthQueryGraphWorkBranchAffinity,
    WorthQueryGraphWorkManagedRunIdentity, WorthQueryGraphWorkSessionIdentity,
};

mod affinity_access;

use crate::domain_computation::execution_runtime::WorthQueryRuntimeAuthorityIdentity;

mod inspection_preparation;
pub mod inspection_read;
mod managed_index_read;
mod mutation;
mod mutation_readmission;
pub use managed_index_read::{
    ManagedIndexReadSource, WorthQueryManagedIndexReadSession,
    WorthQueryManagedIndexSelectionProof, WorthQueryManagedIndexSessionDenial,
};
use mutation::MutationProductBasis;
pub mod prepared_query_read;
mod product_read;
mod query_preparation;
mod query_read;
pub use inspection_read::WorthQuerySessionInspectionReadProof;
mod resource_preparation;
mod resource_read;
pub use prepared_query_read::{
    WorthQueryPreparedQueryPin, WorthQueryPreparedReadCompletion, WorthQueryPreparedSessionRead,
};
pub use product_read::WorthQueryPreparedProductReadSessionIdentity;
use product_read::{ProductReadBasis, ProductReadIntent};
pub use query_read::WorthQueryQuerySourceContext;
pub use resource_read::WorthQuerySessionResourceReadProof;

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum WorthQueryGraphWorkAccessContextAffinity {
    Entity(EntityId),
    InstalledCapability([u8; 32]),
    GovernedEntity {
        entity: EntityId,
        installed_capability: [u8; 32],
    },
}

impl WorthQueryGraphWorkAccessContextAffinity {
    pub const fn entity(entity: EntityId) -> Self {
        Self::Entity(entity)
    }

    pub const fn installed_capability(identity: [u8; 32]) -> Self {
        Self::InstalledCapability(identity)
    }

    pub const fn governed_entity(entity: EntityId, installed_capability: [u8; 32]) -> Self {
        Self::GovernedEntity {
            entity,
            installed_capability,
        }
    }
}

enum WorthQueryGraphWorkBasis {
    ProductRead {
        intent: ProductReadIntent,
        read: ProductReadBasis,
    },
    Mutation(MutationProductBasis),
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum WorthQueryManagedGraphWorkSessionStartDenial {
    PlanAffinityMismatch,
    BasisBranchMismatch,
    IdentityExhausted,
    ManagedRunIdentityExhausted,
}

/// Live authority opened from exactly one sealed, capacity-reserved plan.
pub struct WorthQueryManagedGraphWorkSession {
    identity: WorthQueryGraphWorkSessionIdentity,
    managed_run: WorthQueryGraphWorkManagedRunIdentity,
    runtime: WorthQueryRuntimeAuthorityIdentity,
    binding: ApplicationSchemaBindingIdentity,
    obligation: WorthQueryInstalledGraphObligationSetIdentity,
    subject_authority: String,
    principal: EntityId,
    access: WorthQueryGraphWorkAccessContextAffinity,
    branch: WorthQueryGraphWorkBranchAffinity,
    authorization_branch: WorthQueryGraphWorkBranchAffinity,
    basis: WorthQueryGraphWorkBasis,
    provider: String,
    plan: WorthQueryAdmittedGraphWorkPlan,
    retained_decision_facts: usize,
}

impl WorthQueryManagedGraphWorkSession {
    #[allow(clippy::too_many_arguments)]
    fn start(
        identity: WorthQueryGraphWorkSessionIdentity,
        reserved_managed_run: Option<WorthQueryGraphWorkManagedRunIdentity>,
        plan: WorthQueryAdmittedGraphWorkPlan,
        runtime: WorthQueryRuntimeAuthorityIdentity,
        binding: &ApplicationSchemaBindingIdentity,
        obligation: &WorthQueryInstalledGraphObligationSetIdentity,
        subject_authority: &str,
        principal: EntityId,
        access: WorthQueryGraphWorkAccessContextAffinity,
        branch: WorthQueryGraphWorkBranchAffinity,
        authorization_branch: WorthQueryGraphWorkBranchAffinity,
        basis: WorthQueryGraphWorkBasis,
        provider: &str,
    ) -> Result<Self, WorthQueryManagedGraphWorkSessionStartDenial> {
        if plan.binding_identity() != binding || plan.obligation_identity() != obligation {
            return Err(WorthQueryManagedGraphWorkSessionStartDenial::PlanAffinityMismatch);
        }
        let managed_run = match reserved_managed_run {
            Some(identity) => identity,
            None => WorthQueryGraphWorkManagedRunIdentity::mint()
                .ok_or(WorthQueryManagedGraphWorkSessionStartDenial::ManagedRunIdentityExhausted)?,
        };
        let session = Self {
            identity,
            managed_run,
            runtime,
            binding: binding.clone(),
            obligation: obligation.clone(),
            subject_authority: subject_authority.to_owned(),
            principal,
            access,
            branch,
            authorization_branch,
            basis,
            provider: provider.to_owned(),
            plan,
            retained_decision_facts: 0,
        };
        if !session.retained_affinity_is_exact() {
            return Err(WorthQueryManagedGraphWorkSessionStartDenial::PlanAffinityMismatch);
        }
        Ok(session)
    }

    pub const fn identity(&self) -> WorthQueryGraphWorkSessionIdentity {
        self.identity
    }

    pub const fn managed_run_identity(&self) -> WorthQueryGraphWorkManagedRunIdentity {
        self.managed_run
    }

    pub const fn plan_identity(
        &self,
    ) -> worth_query_admission::facade::graph_obligation::WorthQueryGraphWorkPlanIdentity {
        self.plan.identity()
    }

    pub fn mutation_snapshot(
        &self,
    ) -> Option<&worth_relational::facade::snapshots::SnapshotHandle> {
        match &self.basis {
            WorthQueryGraphWorkBasis::ProductRead { .. } => None,
            WorthQueryGraphWorkBasis::Mutation(basis) => {
                basis.lease.as_deref().map(|l| l.snapshot())
            }
        }
    }

    pub fn take_mutation_lease(&mut self) -> Option<Box<dyn crate::ports::PrimaryMutationLease>> {
        match &mut self.basis {
            WorthQueryGraphWorkBasis::ProductRead { .. } => None,
            WorthQueryGraphWorkBasis::Mutation(basis) => basis.lease.take(),
        }
    }

    pub fn mutation_lease_mut(
        &mut self,
    ) -> Option<&mut Box<dyn crate::ports::PrimaryMutationLease>> {
        match &mut self.basis {
            WorthQueryGraphWorkBasis::ProductRead { .. } => None,
            WorthQueryGraphWorkBasis::Mutation(basis) => basis.lease.as_mut(),
        }
    }

    pub fn mutation_lease(&self) -> Option<&dyn crate::ports::PrimaryMutationLease> {
        match &self.basis {
            WorthQueryGraphWorkBasis::ProductRead { .. } => None,
            WorthQueryGraphWorkBasis::Mutation(basis) => basis.lease.as_deref(),
        }
    }

    pub fn mutation_product(&self) -> Option<&crate::basis::WorthQueryProductBranchLease> {
        self.mutation_lease().map(|l| l.product())
    }

    /// Query truth can remain historical while authorization uses the exact
    /// security Product supplied when this genuine session was admitted.
    pub fn authorization_product(
        &self,
    ) -> Option<&crate::basis::WorthQueryProductObservationLease> {
        match &self.basis {
            WorthQueryGraphWorkBasis::ProductRead { read, .. } => Some(&read.authorization_product),
            WorthQueryGraphWorkBasis::Mutation(basis) => basis.authorization_product(),
        }
    }

    pub fn take_operation_capacity(
        &mut self,
    ) -> Option<worth_query_admission::integration::WorthQueryCapacityReservedExecutionResourcePlan>
    {
        self.plan.take_operation_capacity()
    }

    pub fn record_decision_facts(&mut self, count: usize) {
        self.retained_decision_facts = self.retained_decision_facts.saturating_add(count);
    }

    pub fn set_retained_decision_facts(&mut self, count: usize) {
        self.retained_decision_facts = count;
    }

    pub const fn retained_decision_facts(&self) -> usize {
        self.retained_decision_facts
    }

    pub const fn binding(&self) -> &ApplicationSchemaBindingIdentity {
        &self.binding
    }

    pub const fn obligation(&self) -> &WorthQueryInstalledGraphObligationSetIdentity {
        &self.obligation
    }

    pub fn subject_authority(&self) -> &str {
        &self.subject_authority
    }

    fn retained_affinity_is_exact(&self) -> bool {
        let basis_branch = match &self.basis {
            WorthQueryGraphWorkBasis::ProductRead { read, .. } => read.identity.branch_id(),
            WorthQueryGraphWorkBasis::Mutation(basis) => basis.lease.as_deref().map_or_else(
                || self.branch.relational(),
                |lease| lease.snapshot().branch_id(),
            ),
        };
        let access_is_bound = match &self.access {
            WorthQueryGraphWorkAccessContextAffinity::Entity(_) => true,
            WorthQueryGraphWorkAccessContextAffinity::InstalledCapability(_) => true,
            WorthQueryGraphWorkAccessContextAffinity::GovernedEntity { .. } => true,
        };
        self.runtime.as_u64() > 0
            && self.managed_run.as_u64() > 0
            && self.binding == *self.plan.binding_identity()
            && self.obligation == *self.plan.obligation_identity()
            && !self.subject_authority.is_empty()
            && access_is_bound
            && basis_branch == self.branch.relational()
            && !self.provider.is_empty()
    }

    pub fn query_basis(&self) -> Option<&WorthQueryApplicationBasisIdentity> {
        match &self.basis {
            WorthQueryGraphWorkBasis::ProductRead {
                intent: ProductReadIntent::Query,
                read,
            } => Some(&read.identity),
            WorthQueryGraphWorkBasis::ProductRead {
                intent: ProductReadIntent::Resource(_) | ProductReadIntent::Inspection(_),
                ..
            }
            | WorthQueryGraphWorkBasis::Mutation(_) => None,
        }
    }

    pub fn runtime_ordinal(&self) -> u64 {
        self.runtime.as_u64()
    }

    pub const fn runtime_authority(&self) -> WorthQueryRuntimeAuthorityIdentity {
        self.runtime
    }

    pub const fn principal(&self) -> EntityId {
        self.principal
    }

    pub const fn entity_access_context(&self) -> Option<EntityId> {
        match self.access {
            WorthQueryGraphWorkAccessContextAffinity::Entity(entity)
            | WorthQueryGraphWorkAccessContextAffinity::GovernedEntity { entity, .. } => {
                Some(entity)
            }
            WorthQueryGraphWorkAccessContextAffinity::InstalledCapability(_) => None,
        }
    }

    pub const fn capability_access_context(&self) -> Option<[u8; 32]> {
        match self.access {
            WorthQueryGraphWorkAccessContextAffinity::InstalledCapability(identity) => {
                Some(identity)
            }
            WorthQueryGraphWorkAccessContextAffinity::GovernedEntity {
                installed_capability,
                ..
            } => Some(installed_capability),
            WorthQueryGraphWorkAccessContextAffinity::Entity(_) => None,
        }
    }

    pub fn provider(&self) -> &str {
        &self.provider
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum WorthQueryManagedGraphReadDenial {
    RequestStop(worth_relational::facade::transactions::RelationalExecutionDenialCause),
    SnapshotUnavailable,
    IndexNotInstalled(worth_relational::facade::indexes::DerivedIndexId),
    MutationSession,
    ForeignBasis,
    ForeignGraph,
    ForeignReadProof,
    ForeignReadIntent,
}
