//! What one coupled source edit published, without a producer receipt.
use worth_query_host::facade::{
    application_entry::{
        WorthQueryApplicationOutputDemandSettlement, WorthQueryApplicationProgramOutputSettlement,
    },
    primary_graph::{
        WorthQueryApplicationCommitReceipt, WorthQueryApplicationOutputProjectionDenial,
        WorthQueryApplicationTypedOutputCorrespondence,
    },
};

/// A coupled root is produced inside its source edit: that edit's receipt
/// names the one commit, and the settled root keeps the roles it published.
/// No separate producer receipt exists for either.
pub struct Regeneration<Root> {
    source: WorthQueryApplicationCommitReceipt,
    settled: WorthQueryApplicationProgramOutputSettlement<Root>,
}

impl<Root> Regeneration<Root> {
    pub fn from_settled(
        source: WorthQueryApplicationCommitReceipt,
        settled: WorthQueryApplicationProgramOutputSettlement<Root>,
    ) -> Self {
        assert!(
            settled.root_receipt().is_none(),
            "a co-published root issues no separate producer receipt"
        );
        let fulfillment = settled
            .performed_source()
            .expect("a performed source retains its root fulfillment")
            .root_fulfillment();
        assert!(
            fulfillment.matches_source(&source),
            "the root was fulfilled for this exact source edit"
        );
        assert_eq!(
            fulfillment.output_commit(),
            fulfillment.source_commit(),
            "the settled output is the one the source commit published"
        );
        Self { source, settled }
    }

    /// The edit's own receipt: the commit that changed the source and ran the producer.
    pub fn source(&self) -> &WorthQueryApplicationCommitReceipt {
        &self.source
    }

    /// The roles the producer published in that commit. A foreign contract
    /// refuses, so a successful read also names the producer that ran.
    pub fn outputs_of<Contract: 'static>(
        &self,
    ) -> Result<
        WorthQueryApplicationTypedOutputCorrespondence<'_, Contract>,
        WorthQueryApplicationOutputProjectionDenial,
    > {
        self.settled.root_settlement().outputs_of()
    }

    /// A later demand reuses exactly this publication: it selects this commit
    /// and carries no receipt of its own.
    pub fn assert_reused_by<Query>(
        &self,
        reused: &WorthQueryApplicationOutputDemandSettlement<Query>,
    ) {
        assert!(reused.application_commit_receipt().is_none());
        assert_eq!(
            reused.observation().selected_commit(),
            self.source
                .committed_product_publication()
                .composite_commit(),
            "the reused output is selected at the commit that published it"
        );
    }
}
