use worth_relational::facade::branch::RelationalBranchIdentity;
use worth_relational::facade::history::BranchId;
use worth_signal::facade::branch::{ManagedSignalBranchReference, ValidatedSignalBranchName};

use crate::basis::AdmittedCompositeRuntimeWorldBasis;
use crate::identity::{CompositeCommitIdentity, ProductBranchIdentity, ProductBranchIncarnation};

#[path = "custody/registry.rs"]
mod registry;

pub(crate) use registry::{OwnerCreatedComponentCustodyRegistry, ReservedCustodySlot};

/// Which component owner created a branch on this world's behalf.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CustodyComponent {
    Relational,
    Signal,
}

/// The component branch a Runtime World owner asked a component owner to
/// create, together with the exact owner-issued capability needed to retire it.
/// Custody keeps this capability instead of trying to reconstruct authority
/// from a descriptive name during cleanup.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ComponentBranchTarget {
    Relational {
        target: BranchId,
        identity: RelationalBranchIdentity,
    },
    Signal {
        target: ValidatedSignalBranchName,
        reference: ManagedSignalBranchReference,
    },
}

impl ComponentBranchTarget {
    pub const fn component(&self) -> CustodyComponent {
        match self {
            Self::Relational { .. } => CustodyComponent::Relational,
            Self::Signal { .. } => CustodyComponent::Signal,
        }
    }

    pub fn name(&self) -> &str {
        match self {
            Self::Relational { target, .. } => &target.0,
            Self::Signal { target, .. } => target.as_str(),
        }
    }
}

/// One owner-created component branch charged against the installed custody
/// budget. It is evidence of custody, not authority to delete anything.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct OwnerCreatedComponentCustodyRecord {
    product_branch: ProductBranchIdentity,
    incarnation: ProductBranchIncarnation,
    target: ComponentBranchTarget,
}

impl OwnerCreatedComponentCustodyRecord {
    pub(crate) const fn new(
        product_branch: ProductBranchIdentity,
        incarnation: ProductBranchIncarnation,
        target: ComponentBranchTarget,
    ) -> Self {
        Self {
            product_branch,
            incarnation,
            target,
        }
    }

    pub const fn component(&self) -> CustodyComponent {
        self.target.component()
    }

    pub const fn product_branch(&self) -> &ProductBranchIdentity {
        &self.product_branch
    }

    pub const fn incarnation(&self) -> ProductBranchIncarnation {
        self.incarnation
    }

    pub const fn target(&self) -> &ComponentBranchTarget {
        &self.target
    }

    pub fn into_retirement_work(self) -> OwnerRetirementWork {
        match self.target {
            ComponentBranchTarget::Relational { target, identity } => {
                OwnerRetirementWork::RelationalBranchRetirement { target, identity }
            }
            ComponentBranchTarget::Signal { target, reference } => {
                OwnerRetirementWork::SignalBranchRetirement { target, reference }
            }
        }
    }
}

/// Linear cleanup work carrying the exact authority issued by the component
/// owner when the branch was created.
#[derive(Debug, PartialEq, Eq)]
pub enum OwnerRetirementWork {
    RelationalBranchRetirement {
        target: BranchId,
        identity: RelationalBranchIdentity,
    },
    SignalBranchRetirement {
        target: ValidatedSignalBranchName,
        reference: ManagedSignalBranchReference,
    },
}

impl OwnerRetirementWork {
    pub const fn component(&self) -> CustodyComponent {
        match self {
            Self::RelationalBranchRetirement { .. } => CustodyComponent::Relational,
            Self::SignalBranchRetirement { .. } => CustodyComponent::Signal,
        }
    }

    pub fn target_name(&self) -> &str {
        match self {
            Self::RelationalBranchRetirement { target, .. } => &target.0,
            Self::SignalBranchRetirement { target, .. } => target.as_str(),
        }
    }

    /// Whether a composite history row retains the component branch this
    /// cleanup work must retire. The comparison is descriptive and grants no
    /// component-owner authority.
    pub fn matches_component_basis(&self, basis: &AdmittedCompositeRuntimeWorldBasis) -> bool {
        match self {
            Self::RelationalBranchRetirement { identity, .. } => {
                identity == basis.relational_basis().identity()
            }
            Self::SignalBranchRetirement { reference, .. } => {
                reference.targets_basis(basis.signal_basis())
            }
        }
    }
}

/// Terminal artifact of one product-branch retirement.
#[derive(Debug)]
#[must_use = "retirement work is dispatched or reported, never dropped silently"]
pub struct ProductBranchRetirementReport {
    released_product_reference: ProductBranchIdentity,
    retired_head: CompositeCommitIdentity,
    retirement_boundary: Option<CompositeCommitIdentity>,
    owner_retirement_work: Vec<OwnerRetirementWork>,
}

impl ProductBranchRetirementReport {
    pub(crate) const fn new(
        released_product_reference: ProductBranchIdentity,
        retired_head: CompositeCommitIdentity,
        retirement_boundary: Option<CompositeCommitIdentity>,
        owner_retirement_work: Vec<OwnerRetirementWork>,
    ) -> Self {
        Self {
            released_product_reference,
            retired_head,
            retirement_boundary,
            owner_retirement_work,
        }
    }

    pub const fn released_product_reference(&self) -> &ProductBranchIdentity {
        &self.released_product_reference
    }

    pub fn owner_retirement_work(&self) -> &[OwnerRetirementWork] {
        &self.owner_retirement_work
    }

    pub const fn retired_head(&self) -> &CompositeCommitIdentity {
        &self.retired_head
    }

    /// Exact source boundary installed with this branch occurrence. Query may
    /// carry it into cleanup but does not infer it from component identities.
    pub const fn retirement_boundary(&self) -> Option<&CompositeCommitIdentity> {
        self.retirement_boundary.as_ref()
    }

    pub fn into_cleanup_parts(
        self,
    ) -> (
        CompositeCommitIdentity,
        Option<CompositeCommitIdentity>,
        Vec<OwnerRetirementWork>,
    ) {
        (
            self.retired_head,
            self.retirement_boundary,
            self.owner_retirement_work,
        )
    }
}
