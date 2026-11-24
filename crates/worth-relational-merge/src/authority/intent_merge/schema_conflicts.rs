use crate::contracts::schema::data::registry_errors::SchemaRegistryError;
use crate::contracts::transactions::data::outcomes::commit_conflict::{
    CommitConflict, CommitConflictOwnerPort as _,
};
use crate::contracts::transactions::data::outcomes::conflict_class::ConflictClass;

pub(super) fn schema_error_to_commit_conflict(error: SchemaRegistryError) -> CommitConflict {
    CommitConflict::new(ConflictClass::KindSchemaMismatch {
        detail: format!("{error:?}"),
    })
}
