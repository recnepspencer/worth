//! Internal read-only inspection of folded obligation ledger tables for
//! `worth-query-execution`'s obligation history owner.

use std::collections::BTreeMap;

use worth_foundational::facade::{OwnedAsyncObligationIdentity, ProgramSupportObligationIdentity};

pub use super::adoption::fold::AdoptionFold;
pub use super::output_custody::fold::OutputCustodyFold;
pub use super::output_custody::OutputDemandSubject;
use super::{
    QueryConditionalDecision, QueryObligationLedger, QueryOriginalOperationObligation,
    QueryOutputDemandObligation, QueryResourceObligation,
};

pub fn conditional_decision<'a>(
    ledger: &'a QueryObligationLedger,
    key: &[u8; 32],
) -> Option<&'a QueryConditionalDecision> {
    ledger.conditional_decision(key)
}

pub fn adoption(ledger: &QueryObligationLedger) -> &AdoptionFold {
    &ledger.adoption
}

pub fn output(ledger: &QueryObligationLedger) -> &OutputCustodyFold {
    &ledger.output
}

pub fn original_operations(
    ledger: &QueryObligationLedger,
) -> &BTreeMap<ProgramSupportObligationIdentity, QueryOriginalOperationObligation> {
    &ledger.original_operations
}

pub fn resources(
    ledger: &QueryObligationLedger,
) -> &BTreeMap<OwnedAsyncObligationIdentity, QueryResourceObligation> {
    &ledger.resources
}

pub fn output_demand_subject(row: &QueryOutputDemandObligation) -> OutputDemandSubject {
    row.subject()
}
