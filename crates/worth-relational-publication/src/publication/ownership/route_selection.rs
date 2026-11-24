//! A cold current route is selected from independently authenticated current facts.
use contracts::mvcc::publication::application_publication::output_meaning::RequiredOutputPublicationMeaning;
use contracts::mvcc::publication::application_publication::output_meaning::RequiredOutputPublicationMeaningOwnerPort as _;
use contracts::transactions::data::outcomes::transaction_commit_error::TransactionCommitError;
use contracts::transactions::data::primitives::RecordRef;
use worth_execution::ExecutionRequest;
use worth_relational_branch::internal_port::branch::basis::AdmittedRelationalBranchBasis;
use worth_relational_branch::internal_port::history::data::required_output_positions::presence::Posture;
use worth_relational_branch::internal_port::history::data::required_output_positions::RequiredOutputRoutePosition;
use worth_relational_contracts::internal_port as contracts;
pub(crate) fn read_current(
    execution: &ExecutionRequest<'_, '_>,
    basis: &AdmittedRelationalBranchBasis,
    meaning: &RequiredOutputPublicationMeaning,
) -> Result<Option<RequiredOutputRoutePosition>, TransactionCommitError> {
    let reached =
        super::select_current(execution, basis, &[RecordRef::Entity(meaning.source_scope)])?;
    execution
        .run_native_with_checkpoint(|context| {
            for (_, position) in &reached.positions {
                contracts::publication::ownership::read_request::checkpoint(
                    context,
                    (position.meaning.family.len() + meaning.family.len() + 1) as u64,
                )?;
                if position.meaning.same_route(meaning) {
                    return Ok(Some(match position.posture {
                        Posture::Published => {
                            RequiredOutputRoutePosition::Published(position.publication)
                        }
                        Posture::CodecRetired { retirement } => {
                            RequiredOutputRoutePosition::CodecRetired(retirement)
                        }
                    }));
                }
            }
            Ok(None)
        })
        .map_err(contracts::publication::ownership::read_request::request)?
        .0
}
