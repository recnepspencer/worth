//! Physical answers for queued public medium appends.
use super::bytes::ByteJournal;
use worth_relational::facade::durability::DurablePublicationFate;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ScriptedPublication {
    ProvenNoEffect,
    IndeterminateWithoutEffect,
    IndeterminateAfterEffect,
}
impl ScriptedPublication {
    pub(super) fn answer(
        self,
        journal: &mut ByteJournal,
        position: u64,
        bytes: &[u8],
    ) -> DurablePublicationFate<String> {
        match self {
            Self::ProvenNoEffect => DurablePublicationFate::ProvenNoEffect {
                detail: "scripted: the journal refused before appending".to_owned(),
            },
            Self::IndeterminateWithoutEffect => DurablePublicationFate::Indeterminate {
                detail: "scripted: the append's fate is unknown (not appended)".to_owned(),
            },
            Self::IndeterminateAfterEffect => {
                if journal.head().checked_add(1) == Some(position) {
                    assert!(journal.publish(position, bytes));
                }
                DurablePublicationFate::Indeterminate {
                    detail: "scripted: the append's fate is unknown (appended)".to_owned(),
                }
            }
        }
    }
}
