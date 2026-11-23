//! Durable current publication custody; historical payload selection stays in History.
pub(crate) mod authored_retirement;
pub(crate) mod certification;
mod observation;
pub(crate) mod source_scope;
pub use certification::CurrentOwnershipContactsForTest;
mod cold_checkpoint_source;
mod cold_pairing_court;
mod locality_court;
pub use locality_court::CurrentOwnershipLocalityForTest;
mod completion;
pub(crate) mod corruption;
pub(crate) mod evidence;
pub(crate) mod merge_custody;
mod metadata_guard_court;
pub(crate) mod read;
pub(crate) mod record;
pub(crate) mod selection_pairing;
pub(crate) use completion::complete;
pub(crate) use evidence::PublicationEvidence;
pub(crate) use read::{
    linked_selectors, read, read_candidate, read_selected, CurrentPublicationRoster,
};

pub(crate) fn select_current(
    execution: &worth_execution::ExecutionRequest<'_, '_>,
    basis: &crate::branch::basis::AdmittedRelationalBranchBasis,
    records: &[RecordRef],
) -> Result<std::sync::Arc<CurrentPublicationRoster>, TransactionCommitError> {
    let mut answer = None;
    let handoff = execution.run_native_with_checkpoint(|context| {
        answer = Some(read(execution, basis, records, context).map(std::sync::Arc::new));
    });
    match (handoff, answer) {
        (Ok(_), Some(answer)) => answer,
        (Err(_), Some(Err(cause))) => Err(cause),
        (Err(cause), _) => Err(TransactionCommitError::execution(cause.into())),
        (Ok(_), None) => unreachable!("completed Native read selected its roster"),
    }
}

pub(crate) fn reserve_symbols(
    registry: &crate::schema::data::RelationalSchemaRegistry,
    symbols: &mut crate::runtime::state::subsystems::services::symbol_table::PendingSymbolMeanings,
) -> Result<(), crate::authority::mutation::MutationApplyDenial> {
    if registry.current_publication_storage.is_some() {
        use crate::schema::current_publication_ownership::{
            CATALOG, EDGE, LINK, POSITION, PUBLICATION,
        };
        for text in [
            CATALOG,
            PUBLICATION,
            EDGE,
            POSITION,
            LINK,
            "publication",
            "payload",
        ] {
            symbols.reserve(text)?;
        }
    }
    Ok(())
}

use crate::transactions::data::outcomes::transaction_commit_error::TransactionCommitError;
use crate::transactions::data::primitives::RecordRef;

pub(crate) mod read_request;
use read_request::corrupt;
pub(crate) use read_request::{checkpoint, request};

pub(crate) mod route_selection;
