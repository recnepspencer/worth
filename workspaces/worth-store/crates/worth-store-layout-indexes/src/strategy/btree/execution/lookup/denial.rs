use crate::{planning::AccessPlanIdentity, BTreeReplaySourceDenial};
use worth_store_physical_format::{
    InMemoryPhysicalFormatModelDenial, PhysicalRecordSlot, PhysicalReference,
};
use worth_store_physical_isolation::{
    CompactionProtectedReferenceSet, PhysicalReadPlanAdmissionDenial, StablePhysicalReadReceipt,
};

use super::super::{BaselineBTreeExactCounterWitness, BaselineBTreeLookupCounterReceipt};

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum BTreeSeparatorPartitionDenial {
    LeafSlotsNotCanonical,
    LeftChildCrossesSeparator,
    RightChildPrecedesSeparator,
}

impl BTreeSeparatorPartitionDenial {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::LeafSlotsNotCanonical => "leaf_slots_not_canonical",
            Self::LeftChildCrossesSeparator => "left_child_crosses_separator",
            Self::RightChildPrecedesSeparator => "right_child_precedes_separator",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum BaselineBTreeExecutionDenialKind {
    Physical,
    InvalidRootNode,
    InvalidLeafNode,
    InvalidPhysicalReferenceForBTree,
    WrongSelectedOperation,
    StableReadPlan,
    Recovery,
    CounterEnvelope,
    SeparatorPartition(BTreeSeparatorPartitionDenial),
}

impl BaselineBTreeExecutionDenialKind {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Physical => "physical",
            Self::InvalidRootNode => "invalid_root_node",
            Self::InvalidLeafNode => "invalid_leaf_node",
            Self::InvalidPhysicalReferenceForBTree => "invalid_physical_reference",
            Self::WrongSelectedOperation => "wrong_selected_operation",
            Self::StableReadPlan => "stable_read_plan",
            Self::Recovery => "recovery",
            Self::CounterEnvelope => "counter_envelope",
            Self::SeparatorPartition(denial) => denial.as_str(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum BaselineBTreeExecutionDenial {
    Physical(Box<InMemoryPhysicalFormatModelDenial>),
    InvalidRootNode,
    InvalidLeafNode,
    InvalidPhysicalReferenceForBTree,
    WrongSelectedOperation,
    StableReadPlan(PhysicalReadPlanAdmissionDenial),
    Recovery(Box<BTreeReplaySourceDenial>),
    CounterEnvelope(crate::CounterEnvelopeViolation),
    SeparatorPartition(BTreeSeparatorPartitionDenial),
}

impl BaselineBTreeExecutionDenial {
    pub const fn kind(&self) -> BaselineBTreeExecutionDenialKind {
        match self {
            Self::Physical(_) => BaselineBTreeExecutionDenialKind::Physical,
            Self::InvalidRootNode => BaselineBTreeExecutionDenialKind::InvalidRootNode,
            Self::InvalidLeafNode => BaselineBTreeExecutionDenialKind::InvalidLeafNode,
            Self::InvalidPhysicalReferenceForBTree => {
                BaselineBTreeExecutionDenialKind::InvalidPhysicalReferenceForBTree
            }
            Self::WrongSelectedOperation => {
                BaselineBTreeExecutionDenialKind::WrongSelectedOperation
            }
            Self::StableReadPlan(_) => BaselineBTreeExecutionDenialKind::StableReadPlan,
            Self::Recovery(_) => BaselineBTreeExecutionDenialKind::Recovery,
            Self::CounterEnvelope(_) => BaselineBTreeExecutionDenialKind::CounterEnvelope,
            Self::SeparatorPartition(denial) => {
                BaselineBTreeExecutionDenialKind::SeparatorPartition(*denial)
            }
        }
    }
}

impl From<BTreeReplaySourceDenial> for BaselineBTreeExecutionDenial {
    fn from(value: BTreeReplaySourceDenial) -> Self {
        Self::Recovery(Box::new(value))
    }
}

impl From<InMemoryPhysicalFormatModelDenial> for BaselineBTreeExecutionDenial {
    fn from(value: InMemoryPhysicalFormatModelDenial) -> Self {
        Self::Physical(Box::new(value))
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BaselineBTreeLookupAbsence {
    probe_slot: PhysicalRecordSlot,
    selected_leaf: PhysicalReference,
    exact_counters: BaselineBTreeExactCounterWitness,
}

impl BaselineBTreeLookupAbsence {
    pub(in crate::strategy::btree::execution) fn issue(
        probe_slot: PhysicalRecordSlot,
        selected_leaf: PhysicalReference,
        exact_counters: BaselineBTreeExactCounterWitness,
    ) -> Self {
        Self {
            probe_slot,
            selected_leaf,
            exact_counters,
        }
    }

    pub const fn probe_slot(&self) -> PhysicalRecordSlot {
        self.probe_slot
    }

    pub const fn selected_leaf(&self) -> PhysicalReference {
        self.selected_leaf
    }

    pub const fn exact_counters(&self) -> BaselineBTreeExactCounterWitness {
        self.exact_counters
    }
}

#[derive(Debug, PartialEq, Eq)]
pub(in crate::strategy::btree::execution) enum BaselineBTreeLookupObservation {
    Found(BaselineBTreeLookupExecution),
    Absent(BaselineBTreeLookupAbsence),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BaselineBTreeLookupBranch {
    Left,
    Right,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BaselineBTreeReadShape {
    PointLookup,
    RangeLookup,
    PrefixLookup,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BaselineBTreeLookupExecution {
    shape: BaselineBTreeReadShape,
    probe_slot: PhysicalRecordSlot,
    separator_slot: PhysicalRecordSlot,
    branch: BaselineBTreeLookupBranch,
    selected_reference: PhysicalReference,
    exact_counters: BaselineBTreeExactCounterWitness,
}

impl BaselineBTreeLookupExecution {
    pub(in crate::strategy::btree::execution) const fn new(
        shape: BaselineBTreeReadShape,
        probe_slot: PhysicalRecordSlot,
        separator_slot: PhysicalRecordSlot,
        branch: BaselineBTreeLookupBranch,
        selected_reference: PhysicalReference,
        exact_counters: BaselineBTreeExactCounterWitness,
    ) -> Self {
        Self {
            shape,
            probe_slot,
            separator_slot,
            branch,
            selected_reference,
            exact_counters,
        }
    }

    pub const fn shape(&self) -> BaselineBTreeReadShape {
        self.shape
    }

    pub const fn probe_slot(&self) -> PhysicalRecordSlot {
        self.probe_slot
    }

    pub const fn separator_slot(&self) -> PhysicalRecordSlot {
        self.separator_slot
    }

    pub const fn branch(&self) -> BaselineBTreeLookupBranch {
        self.branch
    }

    pub const fn selected_reference(&self) -> PhysicalReference {
        self.selected_reference
    }

    pub const fn exact_counters(&self) -> BaselineBTreeExactCounterWitness {
        self.exact_counters
    }
}

pub(in crate::strategy::btree::execution) fn exact_counters(
    observation: &BaselineBTreeLookupObservation,
) -> BaselineBTreeExactCounterWitness {
    match observation {
        BaselineBTreeLookupObservation::Found(execution) => execution.exact_counters(),
        BaselineBTreeLookupObservation::Absent(absence) => absence.exact_counters(),
    }
}

pub(in crate::strategy::btree::execution) fn issue_counter_receipt(
    observation: &BaselineBTreeLookupObservation,
    plan_binding: &AccessPlanIdentity,
    stable_read: StablePhysicalReadReceipt,
) -> Result<BaselineBTreeLookupCounterReceipt, crate::CounterEnvelopeViolation> {
    BaselineBTreeLookupCounterReceipt::issue(
        plan_binding,
        exact_counters(observation),
        stable_read.counters().plan_allocations(),
    )
}

pub(in crate::strategy::btree::execution) struct StableReadBindings {
    pub(in crate::strategy::btree::execution) receipt: StablePhysicalReadReceipt,
    pub(in crate::strategy::btree::execution) protected: CompactionProtectedReferenceSet,
}
