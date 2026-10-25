use worth_relational::facade::durability::RelationalRecoveryStop;

use crate::checkpoint_record::RelationalStoreCheckpointRecordRefusal;
use crate::home_descriptor::RelationalStoreHomeDescriptorRefusal;
use crate::physical_store::RelationalStorePhysicalRefusal;
use crate::recovered_history::RelationalStoreJournalRefusal;
use crate::recovered_transitions::RelationalStoreTransitionJournalRefusal;

/// Why a home did not open. Every refusal happens before the runtime serves;
/// none of them treats existing data as an empty home.
#[derive(Debug)]
#[non_exhaustive]
pub enum RelationalStoreHomeOpenRefusal {
    /// The location could not be inspected or created.
    Location {
        kind: std::io::ErrorKind,
        detail: String,
    },
    /// The physical Store could not be created, recovered or opened.
    Physical(RelationalStorePhysicalRefusal),
    /// The home's descriptor could not be read or written.
    Descriptor(RelationalStoreHomeDescriptorRefusal),
    /// The home belongs to a different runtime.
    ForeignHome { expected: String, found: String },
    /// The Store holds Relational history but no home descriptor.
    DescriptorMissing,
    /// The selected checkpoint record could not be read.
    Checkpoint(RelationalStoreCheckpointRecordRefusal),
    /// The journal could not be handed to Relational.
    Journal(RelationalStoreJournalRefusal),
    /// The branch transition journal could not be handed to Relational.
    TransitionJournal(RelationalStoreTransitionJournalRefusal),
    /// Relational refused to readmit or rebuild the recovered history.
    /// This includes a declared schema other than the one the recovered main
    /// head records (`RecoveryFailureClass::DeclaredSchemaMismatch`).
    Readmission(RelationalRecoveryStop),
}
