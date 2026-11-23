use crate::schema::data::SchemaRegistryError;
use crate::transactions::data::outcomes::commit_conflict::CommitConflict;
use crate::transactions::data::outcomes::conflict_class::ConflictClass;

pub(super) fn schema_error_to_commit_conflict(error: SchemaRegistryError) -> CommitConflict {
    CommitConflict::new(ConflictClass::KindSchemaMismatch {
        detail: format!("{error:?}"),
    })
}
