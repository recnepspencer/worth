//! One owned-graph preparation for eligibility and progression, with the producer
//! custody Native restores. Custody without its lineage rows never reaches World.
use super::WorthQueryHistoryNavigationDenial as Denial;
use crate::domain_computation::primary_graph::{
    output_lineage::PreparedHistoricalTransport,
    root::application_publication::ApplicationPublication,
};
use worth_relational::facade::{
    branch::RelationalOwnedModelGraphSelection,
    mvcc::{
        BranchBoundRelationalTransaction, PreparedRelationalCommitCandidate,
        RelationalOwnedGraphSelectionPreparationError as GraphPreparation,
        RelationalPreparedOutputPublications, RelationalRequiredOutputPublication,
        RequiredOutputPublication,
    },
    runtime::RelationalRuntime,
};

/// The routes this selection restores at their selected versions. Absence
/// routes retire the displaced publication and transport no row.
pub(super) struct RestoredOutputCustody<'publisher> {
    pub(super) publisher: &'publisher ApplicationPublication,
    pub(super) routes: Vec<RelationalRequiredOutputPublication>,
}

/// Prepared before World: Native's publication custody for the exact
/// candidate envelope and the selected step's rows for every restored route.
pub(super) struct HistoryOutputTransport {
    pub(super) publications: RelationalPreparedOutputPublications,
    pub(super) lineage: PreparedHistoricalTransport,
}

/// Prepare the selection, passing HistoricalSelection custody whenever the
/// installed publisher owns source scopes and Native finds owned producer
/// publications. Otherwise the selection carries none, so a missing ownership
/// substrate still refuses typed at Native coverage.
pub(super) fn prepare<'publisher>(
    execution: &worth_execution::ExecutionRequest<'_, '_>,
    native: &RelationalRuntime,
    publisher: Option<&'publisher ApplicationPublication>,
    transaction: BranchBoundRelationalTransaction,
    graph: RelationalOwnedModelGraphSelection,
) -> Result<
    (
        PreparedRelationalCommitCandidate,
        Option<RestoredOutputCustody<'publisher>>,
    ),
    Denial,
> {
    let mut custody = None;
    if let Some(publisher) = publisher.filter(|publisher| publisher.native.owns_source_scopes()) {
        let selections = native
            .prepare_historical_output_selections(execution, &publisher.native, &graph)
            .map_err(|cause| Denial::GraphPreparation(GraphPreparation::Preparation(cause)))?;
        if !selections.is_empty() {
            let admission = publisher
                .owner
                .admit_historical_output_selection(
                    selections
                        .iter()
                        .map(|selection| selection.publication().authority()),
                )
                .ok_or(Denial::Document(
                    super::super::WorthQueryDocumentHistoryDenial::InstalledBindingMismatch,
                ))?;
            let routes = selections
                .iter()
                .filter(|selection| !selection.selects_absence())
                .map(|selection| selection.publication().clone())
                .collect();
            custody = Some((
                RequiredOutputPublication::HistoricalSelection {
                    admission,
                    selections,
                },
                RestoredOutputCustody { publisher, routes },
            ));
        }
    }
    let port = native.preparation_port();
    let selection = port
        .admit_owned_graph_selection(execution, transaction, graph)
        .map_err(Denial::GraphPreparation)?;
    let Some((publication, restored)) = custody else {
        return port
            .prepare_owned_graph_selection(execution, selection, None)
            .map(|prepared| (prepared, None))
            .map_err(Denial::GraphPreparation);
    };
    port.prepare_owned_graph_selection(
        execution,
        selection,
        Some((&restored.publisher.native, publication)),
    )
    .map(|prepared| (prepared, Some(restored)))
    .map_err(Denial::GraphPreparation)
}
