//! A failed funded transition retains Bank custody and the exact native refusal.
use super::BankCommitRecoveryHandle;
use crate::{BankRecoveryDenial, BankRecoveryDenialKind};
#[derive(Debug)]
pub struct BankRecoveryTransitionDenial {
    query: worth_query_host::facade::primary_graph::WorthQueryRecoveryTransitionDenial,
}
impl BankRecoveryTransitionDenial {
    pub(crate) fn from_query(
        query: worth_query_host::facade::primary_graph::WorthQueryRecoveryTransitionDenial,
    ) -> Self {
        Self { query }
    }
    pub fn kind(&self) -> BankRecoveryDenialKind {
        BankRecoveryDenial::from_query(self.query.denial().clone()).kind()
    }
    pub fn source(
        &self,
    ) -> &worth_query_host::facade::primary_graph::WorthQueryRecoveryHandleDenial {
        self.query.denial()
    }
    pub fn into_parts(self) -> (BankRecoveryDenial, Option<BankCommitRecoveryHandle>) {
        let (denial, handle) = self.query.into_parts();
        (
            BankRecoveryDenial::from_query(denial),
            handle.map(|query| BankCommitRecoveryHandle { query }),
        )
    }
}
