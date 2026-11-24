//! Bind the installed runtime identity to Publication's historical selection.

use super::RelationalRuntime;
use crate::branch::basis::AdmittedRelationalBranchBasis;
use crate::branch::model_ownership::RelationalOwnedModelGraphSelection;
use crate::mvcc::publication::application_publication::{
    historical_selection, output_meaning::RequiredOutputPublicationMeaning,
    RelationalHistoricalOutputSelection, RelationalInstalledApplicationPublicationAuthority,
};
use crate::transactions::data::outcomes::transaction_commit_error::TransactionCommitError;
use worth_execution::ExecutionRequest;

impl RelationalRuntime {
    pub fn prepare_historical_output_selection(
        &self,
        execution: &ExecutionRequest<'_, '_>,
        installed: &RelationalInstalledApplicationPublicationAuthority,
        current: &AdmittedRelationalBranchBasis,
        selected: &AdmittedRelationalBranchBasis,
        route: &RequiredOutputPublicationMeaning,
    ) -> Result<RelationalHistoricalOutputSelection, TransactionCommitError> {
        historical_selection::prepare_historical_output_selection(
            self.runtime_instance_id(),
            execution,
            installed,
            current,
            selected,
            route,
        )
    }

    pub fn prepare_historical_output_selections(
        &self,
        execution: &ExecutionRequest<'_, '_>,
        installed: &RelationalInstalledApplicationPublicationAuthority,
        graph: &RelationalOwnedModelGraphSelection,
    ) -> Result<Vec<RelationalHistoricalOutputSelection>, TransactionCommitError> {
        historical_selection::graph::prepare_historical_output_selections(
            self.runtime_instance_id(),
            execution,
            installed,
            graph,
        )
    }
}
