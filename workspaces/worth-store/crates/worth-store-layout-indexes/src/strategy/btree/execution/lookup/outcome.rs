use super::{
    BTreeLookupExecutionCaseId, BaselineBTreeExecutionDenial, BaselineBTreeLookupAbsence,
    BaselineBTreeLookupExecution, BaselineBTreeLookupObservation, StableReadBindings,
};
use crate::planning::AccessPlanIdentity;

#[derive(Debug, PartialEq, Eq)]
pub struct StableBTreeLookupExecution {
    observation: BaselineBTreeLookupObservation,
    stable_read: worth_store_physical_isolation::PhysicalReadPlanCompletionReceipt,
    protected: worth_store_physical_isolation::CompactionProtectedReferenceSet,
    current_materialization: crate::CurrentLayoutMaterialization,
    counter_receipt: super::super::BaselineBTreeLookupCounterReceipt,
}

impl StableBTreeLookupExecution {
    pub(in crate::strategy::btree::execution) fn issue(
        observation: BaselineBTreeLookupObservation,
        plan_binding: &AccessPlanIdentity,
        stable_read: StableReadBindings,
        current_materialization: crate::CurrentLayoutMaterialization,
    ) -> Result<Self, crate::CounterEnvelopeViolation> {
        let counter_receipt =
            super::denial::issue_counter_receipt(&observation, plan_binding, stable_read.receipt)?;
        Ok(Self {
            observation,
            stable_read: stable_read.receipt,
            protected: stable_read.protected,
            current_materialization,
            counter_receipt,
        })
    }

    pub const fn read_plan_completion(
        &self,
    ) -> &worth_store_physical_isolation::PhysicalReadPlanCompletionReceipt {
        &self.stable_read
    }

    pub const fn protected(
        &self,
    ) -> &worth_store_physical_isolation::CompactionProtectedReferenceSet {
        &self.protected
    }

    pub const fn current_materialization(&self) -> &crate::CurrentLayoutMaterialization {
        &self.current_materialization
    }

    pub const fn counter_receipt(&self) -> &super::super::BaselineBTreeLookupCounterReceipt {
        &self.counter_receipt
    }

    pub const fn view(&self) -> BTreeLookupExecutionView<'_> {
        match &self.observation {
            BaselineBTreeLookupObservation::Found(found) => BTreeLookupExecutionView::Found(found),
            BaselineBTreeLookupObservation::Absent(absent) => {
                BTreeLookupExecutionView::Absent(absent)
            }
        }
    }
}

#[derive(Debug, PartialEq, Eq)]
enum BTreeLookupExecutionCase {
    Executed(Box<StableBTreeLookupExecution>),
    Denied(BaselineBTreeExecutionDenial),
}

#[derive(Debug, PartialEq, Eq)]
pub struct BTreeLookupExecutionOutcome {
    case: BTreeLookupExecutionCase,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BTreeLookupExecutionView<'a> {
    Found(&'a BaselineBTreeLookupExecution),
    Absent(&'a BaselineBTreeLookupAbsence),
    Denied(&'a BaselineBTreeExecutionDenial),
}

impl BTreeLookupExecutionOutcome {
    pub(super) fn issue(
        result: Result<StableBTreeLookupExecution, BaselineBTreeExecutionDenial>,
    ) -> Self {
        Self {
            case: match result {
                Ok(executed) => BTreeLookupExecutionCase::Executed(Box::new(executed)),
                Err(denial) => BTreeLookupExecutionCase::Denied(denial),
            },
        }
    }

    pub const fn view(&self) -> BTreeLookupExecutionView<'_> {
        match &self.case {
            BTreeLookupExecutionCase::Executed(executed) => match &executed.observation {
                BaselineBTreeLookupObservation::Found(found) => {
                    BTreeLookupExecutionView::Found(found)
                }
                BaselineBTreeLookupObservation::Absent(absent) => {
                    BTreeLookupExecutionView::Absent(absent)
                }
            },
            BTreeLookupExecutionCase::Denied(denial) => BTreeLookupExecutionView::Denied(denial),
        }
    }

    pub const fn counter_receipt(
        &self,
    ) -> Option<&super::super::BaselineBTreeLookupCounterReceipt> {
        match &self.case {
            BTreeLookupExecutionCase::Executed(executed) => Some(executed.counter_receipt()),
            BTreeLookupExecutionCase::Denied(_) => None,
        }
    }

    pub const fn execution(&self) -> Option<&StableBTreeLookupExecution> {
        match &self.case {
            BTreeLookupExecutionCase::Executed(executed) => Some(executed),
            BTreeLookupExecutionCase::Denied(_) => None,
        }
    }

    pub const fn case_id(&self) -> BTreeLookupExecutionCaseId {
        self.case_id_internal()
    }

    pub fn into_result(self) -> Result<StableBTreeLookupExecution, BaselineBTreeExecutionDenial> {
        match self.case {
            BTreeLookupExecutionCase::Executed(executed) => Ok(*executed),
            BTreeLookupExecutionCase::Denied(denial) => Err(denial),
        }
    }

    pub(super) const fn case_id_internal(&self) -> BTreeLookupExecutionCaseId {
        match &self.case {
            BTreeLookupExecutionCase::Executed(executed) => match executed.observation {
                BaselineBTreeLookupObservation::Found(_) => BTreeLookupExecutionCaseId::Found,
                BaselineBTreeLookupObservation::Absent(_) => BTreeLookupExecutionCaseId::Absent,
            },
            BTreeLookupExecutionCase::Denied(denial) => {
                BTreeLookupExecutionCaseId::Denied(denial.kind())
            }
        }
    }
}
