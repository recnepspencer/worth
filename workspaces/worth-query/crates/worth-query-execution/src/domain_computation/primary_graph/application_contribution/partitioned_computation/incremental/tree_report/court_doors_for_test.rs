//! Sealed test doors over this module's private members.
use super::*;

impl<Tree, Stopped> ReportedTree<Tree, Stopped> {
    #[doc(hidden)]
    pub fn outcome_for_test(
        &self,
    ) -> &Result<Tree, WorthQueryPartitionedComputationDenial<Stopped>> {
        &self.outcome
    }
}

impl<Tree, Stopped> ReportedTree<Tree, Stopped> {
    #[doc(hidden)]
    pub fn report_for_test(&self) -> &WorthQueryPartitionedTreeRun {
        &self.report
    }
}

impl<Tree, Stopped> ReportedTree<Tree, Stopped> {
    #[doc(hidden)]
    pub fn outcome_into_for_test(
        self,
    ) -> Result<Tree, WorthQueryPartitionedComputationDenial<Stopped>> {
        self.outcome
    }
}
