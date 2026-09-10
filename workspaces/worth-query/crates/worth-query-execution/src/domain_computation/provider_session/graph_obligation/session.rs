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
use crate::domain_computation::execution_runtime::WorthQueryRuntimeAuthorityIdentity;
use crate::domain_computation::primary_graph::WorthQueryApplicationSnapshotLease;

mod query_read;

#[derive(Clone, Debug, Eq, PartialEq)]
pub(in crate::domain_computation) enum WorthQueryGraphWorkAccessContextAffinity {
    Entity(EntityId),
    InstalledCapability([u8; 32]),
    GovernedEntity {
        entity: EntityId,
        installed_capability: [u8; 32],
    },
}

impl WorthQueryGraphWorkAccessContextAffinity {
    pub(in crate::domain_computation) const fn entity(entity: EntityId) -> Self {
        Self::Entity(entity)
    }

    pub(in crate::domain_computation) const fn installed_capability(identity: [u8; 32]) -> Self {
        Self::InstalledCapability(identity)
    }

    pub(in crate::domain_computation) const fn governed_entity(
        entity: EntityId,
        installed_capability: [u8; 32],
    ) -> Self {
        Self::GovernedEntity {
            entity,
            installed_capability,
        }
    }
}

enum WorthQueryGraphWorkBasis {
    Query {
        identity: WorthQueryApplicationBasisIdentity,
        product: crate::basis::WorthQueryProductObservationLease,
        port: WorthQueryGraphReadOwnerPort,
    },
    Mutation(Option<WorthQueryApplicationSnapshotLease>),
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(in crate::domain_computation) enum WorthQueryManagedGraphWorkSessionStartDenial {
    PlanAffinityMismatch,
    BasisBranchMismatch,
    IdentityExhausted,
    ManagedRunIdentityExhausted,
}

/// Live authority opened from exactly one sealed, capacity-reserved plan.
pub(in crate::domain_computation) struct WorthQueryManagedGraphWorkSession {
    identity: WorthQueryGraphWorkSessionIdentity,
    managed_run: WorthQueryGraphWorkManagedRunIdentity,
    runtime: WorthQueryRuntimeAuthorityIdentity,
    binding: ApplicationSchemaBindingIdentity,
    obligation: WorthQueryInstalledGraphObligationSetIdentity,
    subject_authority: String,
    principal: EntityId,
    access: WorthQueryGraphWorkAccessContextAffinity,
    branch: WorthQueryGraphWorkBranchAffinity,
    basis: WorthQueryGraphWorkBasis,
    provider: String,
    plan: WorthQueryAdmittedGraphWorkPlan,
    retained_decision_facts: usize,
}

impl WorthQueryManagedGraphWorkSession {
    #[allow(clippy::too_many_arguments)]
    pub(in crate::domain_computation) fn start_query(
        plan: WorthQueryAdmittedGraphWorkPlan,
        runtime: WorthQueryRuntimeAuthorityIdentity,
        binding: &ApplicationSchemaBindingIdentity,
        obligation: &WorthQueryInstalledGraphObligationSetIdentity,
        subject_authority: &str,
        principal: EntityId,
        access: WorthQueryGraphWorkAccessContextAffinity,
        basis: &WorthQueryApplicationBasisIdentity,
        product: crate::basis::WorthQueryProductObservationLease,
        provider: &str,
        port: WorthQueryGraphReadOwnerPort,
    ) -> Result<Self, WorthQueryManagedGraphWorkSessionStartDenial> {
        let branch = WorthQueryGraphWorkBranchAffinity::from_query_basis(basis);
        let selected_product = match basis.selection() {
            crate::domain_computation::primary_graph::WorthQueryApplicationBasisSelectionIdentity::Product(identity) => identity,
            crate::domain_computation::primary_graph::WorthQueryApplicationBasisSelectionIdentity::Relational => {
                return Err(WorthQueryManagedGraphWorkSessionStartDenial::BasisBranchMismatch);
            }
        };
        let carried_product = crate::basis::WorthQueryProductBranchReadIdentity::from_observation(
            product.observation(),
        );
        if !branch.admits_query_basis(basis) || selected_product != &carried_product {
            return Err(WorthQueryManagedGraphWorkSessionStartDenial::BasisBranchMismatch);
        }
        Self::start(
            plan,
            runtime,
            binding,
            obligation,
            subject_authority,
            principal,
            access,
            branch,
            WorthQueryGraphWorkBasis::Query {
                identity: basis.clone(),
                product,
                port,
            },
            provider,
        )
    }

    #[allow(clippy::too_many_arguments)]
    pub(in crate::domain_computation) fn start_mutation(
        plan: WorthQueryAdmittedGraphWorkPlan,
        runtime: WorthQueryRuntimeAuthorityIdentity,
        binding: &ApplicationSchemaBindingIdentity,
        obligation: &WorthQueryInstalledGraphObligationSetIdentity,
        subject_authority: &str,
        principal: EntityId,
        access: WorthQueryGraphWorkAccessContextAffinity,
        lease: WorthQueryApplicationSnapshotLease,
        provider: &str,
    ) -> Result<Self, WorthQueryManagedGraphWorkSessionStartDenial> {
        let branch = WorthQueryGraphWorkBranchAffinity::from_snapshot(lease.snapshot());
        if !branch.admits_snapshot(lease.snapshot()) {
            return Err(WorthQueryManagedGraphWorkSessionStartDenial::BasisBranchMismatch);
        }
        Self::start(
            plan,
            runtime,
            binding,
            obligation,
            subject_authority,
            principal,
            access,
            branch,
            WorthQueryGraphWorkBasis::Mutation(Some(lease)),
            provider,
        )
    }

    #[allow(clippy::too_many_arguments)]
    fn start(
        plan: WorthQueryAdmittedGraphWorkPlan,
        runtime: WorthQueryRuntimeAuthorityIdentity,
        binding: &ApplicationSchemaBindingIdentity,
        obligation: &WorthQueryInstalledGraphObligationSetIdentity,
        subject_authority: &str,
        principal: EntityId,
        access: WorthQueryGraphWorkAccessContextAffinity,
        branch: WorthQueryGraphWorkBranchAffinity,
        basis: WorthQueryGraphWorkBasis,
        provider: &str,
    ) -> Result<Self, WorthQueryManagedGraphWorkSessionStartDenial> {
        if plan.binding_identity() != binding || plan.obligation_identity() != obligation {
            return Err(WorthQueryManagedGraphWorkSessionStartDenial::PlanAffinityMismatch);
        }
        let identity = WorthQueryGraphWorkSessionIdentity::mint()
            .ok_or(WorthQueryManagedGraphWorkSessionStartDenial::IdentityExhausted)?;
        let managed_run = WorthQueryGraphWorkManagedRunIdentity::mint()
            .ok_or(WorthQueryManagedGraphWorkSessionStartDenial::ManagedRunIdentityExhausted)?;
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

    pub(in crate::domain_computation) const fn identity(
        &self,
    ) -> WorthQueryGraphWorkSessionIdentity {
        self.identity
    }

    pub(in crate::domain_computation) const fn managed_run_identity(
        &self,
    ) -> WorthQueryGraphWorkManagedRunIdentity {
        self.managed_run
    }

    pub(in crate::domain_computation) const fn plan_identity(
        &self,
    ) -> worth_query_admission::facade::graph_obligation::WorthQueryGraphWorkPlanIdentity {
        self.plan.identity()
    }

    pub(in crate::domain_computation) const fn branch(&self) -> &WorthQueryGraphWorkBranchAffinity {
        &self.branch
    }

    pub(in crate::domain_computation) fn admits_snapshot(
        &self,
        snapshot: &worth_relational::facade::snapshots::SnapshotHandle,
    ) -> bool {
        self.branch.admits_snapshot(snapshot)
    }

    pub(in crate::domain_computation) fn mutation_snapshot(
        &self,
    ) -> Option<&worth_relational::facade::snapshots::SnapshotHandle> {
        match &self.basis {
            WorthQueryGraphWorkBasis::Query { .. } => None,
            WorthQueryGraphWorkBasis::Mutation(lease) => lease
                .as_ref()
                .map(WorthQueryApplicationSnapshotLease::snapshot),
        }
    }

    pub(in crate::domain_computation) fn mutation_handle(
        &self,
    ) -> Option<&crate::domain_computation::primary_graph::WorthQueryPrimaryGraphIntegrationHandle>
    {
        match &self.basis {
            WorthQueryGraphWorkBasis::Query { .. } => None,
            WorthQueryGraphWorkBasis::Mutation(lease) => lease
                .as_ref()
                .map(WorthQueryApplicationSnapshotLease::handle),
        }
    }

    pub(in crate::domain_computation) fn take_mutation_lease(
        &mut self,
    ) -> Option<WorthQueryApplicationSnapshotLease> {
        match &mut self.basis {
            WorthQueryGraphWorkBasis::Query { .. } => None,
            WorthQueryGraphWorkBasis::Mutation(lease) => lease.take(),
        }
    }

    pub(in crate::domain_computation) fn mutation_lease(
        &self,
    ) -> Option<&WorthQueryApplicationSnapshotLease> {
        match &self.basis {
            WorthQueryGraphWorkBasis::Query { .. } => None,
            WorthQueryGraphWorkBasis::Mutation(lease) => lease.as_ref(),
        }
    }

    pub(in crate::domain_computation) fn mutation_product(
        &self,
    ) -> Option<&crate::basis::WorthQueryProductBranchLease> {
        self.mutation_lease()
            .map(WorthQueryApplicationSnapshotLease::product)
    }

    pub(in crate::domain_computation) fn product(
        &self,
    ) -> Option<&crate::basis::WorthQueryProductObservationLease> {
        match &self.basis {
            WorthQueryGraphWorkBasis::Query { product, .. } => Some(product),
            WorthQueryGraphWorkBasis::Mutation(lease) => {
                lease.as_ref().map(|lease| lease.product().read_lease_ref())
            }
        }
    }

    pub(in crate::domain_computation) fn take_operation_capacity(
        &mut self,
    ) -> Option<worth_query_admission::integration::WorthQueryCapacityReservedExecutionResourcePlan>
    {
        self.plan.take_operation_capacity()
    }

    pub(in crate::domain_computation) fn record_decision_facts(&mut self, count: usize) {
        self.retained_decision_facts = self.retained_decision_facts.saturating_add(count);
    }

    pub(in crate::domain_computation) fn set_retained_decision_facts(&mut self, count: usize) {
        self.retained_decision_facts = count;
    }

    pub(in crate::domain_computation) const fn retained_decision_facts(&self) -> usize {
        self.retained_decision_facts
    }

    pub(in crate::domain_computation) const fn binding(&self) -> &ApplicationSchemaBindingIdentity {
        &self.binding
    }

    pub(in crate::domain_computation) const fn obligation(
        &self,
    ) -> &WorthQueryInstalledGraphObligationSetIdentity {
        &self.obligation
    }

    pub(in crate::domain_computation) fn subject_authority(&self) -> &str {
        &self.subject_authority
    }

    fn retained_affinity_is_exact(&self) -> bool {
        let basis_branch = match &self.basis {
            WorthQueryGraphWorkBasis::Query { identity, .. } => identity.branch_id(),
            WorthQueryGraphWorkBasis::Mutation(Some(lease)) => lease.snapshot().branch_id(),
            WorthQueryGraphWorkBasis::Mutation(None) => self.branch.relational(),
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

    pub(in crate::domain_computation) fn query_basis(
        &self,
    ) -> Option<&WorthQueryApplicationBasisIdentity> {
        match &self.basis {
            WorthQueryGraphWorkBasis::Query { identity, .. } => Some(identity),
            WorthQueryGraphWorkBasis::Mutation(_) => None,
        }
    }

    pub(in crate::domain_computation) fn runtime_ordinal(&self) -> u64 {
        self.runtime.as_u64()
    }

    pub(in crate::domain_computation) const fn runtime_authority(
        &self,
    ) -> WorthQueryRuntimeAuthorityIdentity {
        self.runtime
    }

    pub(in crate::domain_computation) const fn principal(&self) -> EntityId {
        self.principal
    }

    pub(in crate::domain_computation) const fn entity_access_context(&self) -> Option<EntityId> {
        match self.access {
            WorthQueryGraphWorkAccessContextAffinity::Entity(entity)
            | WorthQueryGraphWorkAccessContextAffinity::GovernedEntity { entity, .. } => {
                Some(entity)
            }
            WorthQueryGraphWorkAccessContextAffinity::InstalledCapability(_) => None,
        }
    }

    pub(in crate::domain_computation) const fn capability_access_context(
        &self,
    ) -> Option<[u8; 32]> {
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

    pub(in crate::domain_computation) fn provider(&self) -> &str {
        &self.provider
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(in crate::domain_computation) enum WorthQueryManagedGraphReadDenial {
    MutationSession,
    ForeignBasis,
    ForeignGraph,
    ForeignReadProof,
    TerminalReleaseMismatch,
}
