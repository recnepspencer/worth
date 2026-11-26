//! The reversal facts live on the document's branch (public #56, ruling (u)),
//! for the DST's reversal reader: each recorded correction step whose posted
//! aftermath-causality fact is live at the head, with the step the fact says
//! it reversed resolved through the fact's own source commit.
use super::*;
use worth_query_execution_attempt::facade::ports::PrimaryGraphProviderInternalPort as _;
use worth_query_execution_lineage::facade::ports::ReadObservationInternalPort as _;
use worth_query_execution_query::facade::ports::ApplicationRuntimeCoreInternalPort as _;
use worth_query_execution_runtime::facade::ports::ProductBranchLeaseInternalPort as _;

/// One live reversal fact.
#[doc(hidden)]
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct WorthQueryHistoryReversalFactForTest {
    /// The correction step that posted the fact.
    pub step: WorthQueryHistoryNodeId,
    /// The recorded step whose commit the fact names as its source, `None`
    /// when no recorded step made that commit.
    pub source: Option<WorthQueryHistoryNodeId>,
    /// The record the correction reversed.
    pub record: worth_relational::facade::transactions::RecordRef,
}

impl<S: ApplicationSchema, H: ApplicationHistoryBinding<S>> WorthQueryDocumentHistory<'_, S, H> {
    /// Rewrites the fact `step` posted in an ordinary commit (no navigation)
    /// and whether the canonical read still accepts it there: ruling (u)'s
    /// read-trust negative court. The document is spent afterwards.
    #[doc(hidden)]
    pub fn rewrite_posted_fact_for_test(
        &self,
        phase: &crate::domain_computation::primary_graph::WorthQueryAdvancementPhase<'_>,
        step: &WorthQueryHistoryNodeId,
    ) -> Result<(), crate::domain_computation::primary_graph::provider::WorthQueryAftermathCausalityReadDenial>
    {
        let (fact, commit) = {
            let selected = self.selection.lock();
            selected
                .journal
                .posted_facts()
                .into_iter()
                .find(|(posted, _, _)| posted == step)
                .map(|(_, fact, commit)| (fact, commit))
                .expect("the step posted a fact")
        };
        let application = self.application();
        let head = application
            .on_branch(self.branch())
            .select(phase)
            .expect("the document head reads");
        let execution = phase
            .checked_execution_request_for(application.inner.runtime.product_runtime_ref())
            .expect("the phase admits a request");
        application
            .inner
            .runtime
            .primary_provider_ref()
            .rewrite_and_read_correction_fact_for_test(
                &execution,
                head.product().relational_basis(),
                fact,
                owner_coordinate(&commit),
            )
            .map(|read| assert!(read.is_some(), "a rewritten fact still reads live"))
    }

    /// Every recorded correction step's fact that is live at the head.
    #[doc(hidden)]
    pub fn reversal_facts_for_test(
        &self,
        phase: &crate::domain_computation::primary_graph::WorthQueryAdvancementPhase<'_>,
    ) -> Vec<WorthQueryHistoryReversalFactForTest> {
        let (posted, sources) = {
            let selected = self.selection.lock();
            (selected.journal.posted_facts(), selected.journal.recorded_sources())
        };
        let application = self.application();
        let head = application
            .on_branch(self.branch())
            .select(phase)
            .expect("the document head reads");
        let execution = phase
            .checked_execution_request_for(application.inner.runtime.product_runtime_ref())
            .expect("the phase admits a request");
        let provider = application.inner.runtime.primary_provider_ref();
        posted
            .into_iter()
            .filter_map(|(step, fact, commit)| {
                let fact = provider
                    .correction_fact_at_basis_for_test(
                        &execution,
                        head.product().relational_basis(),
                        fact,
                        owner_coordinate(&commit),
                    )
                    .expect("a posted fact reads")?;
                let source = sources
                    .iter()
                    .find(|(commit, _)| commit.ordinal() == fact.source.world_commit())
                    .map(|(_, step)| *step);
                Some(WorthQueryHistoryReversalFactForTest { step, source, record: fact.record })
            })
            .collect()
    }
}

/// The world owner coordinate a fact posted by `commit` names.
fn owner_coordinate(
    commit: &worth_runtime_world::facade::CompositeCommitIdentity,
) -> worth_foundational::facade::WorldOwnerCoordinate {
    match commit.owner_identity().durable() {
        Some(owner) => worth_foundational::facade::WorldOwnerCoordinate::Home { owner },
        None => worth_foundational::facade::WorldOwnerCoordinate::MemoryOnly {
            observation: commit.owner_identity().get(),
        },
    }
}
