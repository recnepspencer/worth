//! Durable current publication custody; historical payload selection stays in History.
use contracts::schema::data::RelationalSchemaRegistryFieldsPort as _;
use contracts::transactions::data::outcomes::transaction_commit_error::TransactionCommitErrorOwnerPort as _;
use worth_relational_contracts::internal_port as contracts;
pub mod authored_retirement;
pub mod certification;
pub mod source_scope;
pub use worth_relational_branch::internal_port::publication::ownership::observation::CurrentOwnershipContactsForTest;
pub mod cold_checkpoint_source;
pub mod cold_pairing_court;
pub mod locality_court;
pub use locality_court::CurrentOwnershipLocalityObservation;
pub mod completion;
pub mod merge_custody;
pub mod metadata_guard_court;
pub mod selection_pairing;
pub(crate) use completion::complete;
pub use worth_relational_branch::internal_port::publication::ownership::evidence::PublicationEvidence;
pub(crate) use worth_relational_branch::internal_port::publication::ownership::read::linked::selectors as linked_selectors;
pub use worth_relational_branch::internal_port::publication::ownership::read::roster::CurrentPublicationRoster;
pub(crate) use worth_relational_branch::internal_port::publication::ownership::read::{
    read, read_candidate, read_selected,
};

pub(crate) fn select_current(
    execution: &worth_execution::ExecutionRequest<'_, '_>,
    basis: &worth_relational_branch::internal_port::branch::basis::AdmittedRelationalBranchBasis,
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
    registry: &contracts::schema::data::RelationalSchemaRegistry,
    symbols: &mut worth_relational_storage::internal_port::runtime::state::subsystems::services::symbol_table::reservations::PendingSymbolMeanings,
) -> Result<(), worth_relational_mutation::internal_port::authority::mutation::round_denial::MutationApplyDenial>{
    if (*registry.owner_current_publication_storage()).is_some() {
        use contracts::schema::current_publication_ownership::{
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

use contracts::transactions::data::outcomes::transaction_commit_error::TransactionCommitError;
use contracts::transactions::data::primitives::RecordRef;

pub(crate) use contracts::publication::ownership::read_request::checkpoint;
pub(crate) use contracts::publication::ownership::read_request::request;

pub mod route_selection;
