//! Custody of one handler's total retention result before seal.

use super::{CompletedComputationRetention, PriorAbsence};
use std::sync::{Arc, Mutex};

#[derive(Clone)]
pub(in crate::domain_computation::primary_graph) struct ComputationDeposit(
    Arc<Mutex<DepositCustody>>,
);

enum DepositCustody {
    Held(CompletedComputationRetention),
    Taken,
}

impl ComputationDeposit {
    pub(in crate::domain_computation::primary_graph) fn new() -> Self {
        Self(Arc::new(Mutex::new(DepositCustody::Held(
            CompletedComputationRetention::Absent(PriorAbsence::NotProduced),
        ))))
    }

    pub(in crate::domain_computation::primary_graph) fn write(
        &self,
        result: CompletedComputationRetention,
    ) {
        let mut custody = self
            .0
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        assert!(
            matches!(*custody, DepositCustody::Held(_)),
            "a sealed deposit cannot be rewritten"
        );
        *custody = DepositCustody::Held(result);
    }

    pub(in crate::domain_computation::primary_graph) fn take(
        &self,
    ) -> CompletedComputationRetention {
        match std::mem::replace(
            &mut *self
                .0
                .lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner),
            DepositCustody::Taken,
        ) {
            DepositCustody::Held(result) => result,
            DepositCustody::Taken => panic!("one read completion owns the deposit"),
        }
    }
}
