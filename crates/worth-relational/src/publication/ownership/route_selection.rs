//! A cold current route is selected from independently authenticated current facts.
use crate::branch::basis::AdmittedRelationalBranchBasis;
use crate::history::data::current_route_presence::Posture;
use crate::history::data::RequiredOutputRoutePosition;
use crate::mvcc::publication::application_publication::output_meaning::RequiredOutputPublicationMeaning;
use crate::transactions::data::outcomes::transaction_commit_error::TransactionCommitError;
use crate::transactions::data::primitives::RecordRef;
use worth_execution::ExecutionRequest;
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
                super::read_request::checkpoint(
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
        .map_err(super::read_request::request)?
        .0
}
