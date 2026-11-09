//! A cold current route is selected from independently authenticated current facts.
use super::*;
use crate::{
    branch::AdmittedRelationalBranchBasis,
    history::data::RequiredOutputRoutePosition,
    transactions::data::{RecordRef, TransactionCommitError},
};
use worth_execution::ExecutionRequest;
pub(crate) fn read_current(
    execution: &ExecutionRequest<'_, '_>,
    basis: &AdmittedRelationalBranchBasis,
    meaning: &RequiredOutputPublicationMeaning,
) -> Result<Option<RequiredOutputRoutePosition>, TransactionCommitError> {
    let reached = crate::publication::ownership::select_current(
        execution,
        basis,
        &[RecordRef::Entity(meaning.source_scope)],
    )?;
    execution
        .run_native_with_checkpoint(|context| {
            for (_, position) in &reached.positions {
                crate::publication::ownership::checkpoint(
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
        .map_err(crate::publication::ownership::request)?
        .0
}
