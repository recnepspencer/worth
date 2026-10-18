use std::collections::BTreeSet;

use crate::transactions::data::{
    CommitConflict, ConflictClass, ExistingRecordTarget, MutationIntent,
};

pub(crate) fn detect_conflicting_updates(intents: &[MutationIntent]) -> Result<(), CommitConflict> {
    let mut seen_updates = BTreeSet::new();
    for intent in intents {
        if let Some(target) = intent.existing_record_target()
            && !seen_updates.insert(target)
        {
            let detail = match target {
                ExistingRecordTarget::Entity(entity_id) => {
                    format!(
                        "conflicting entity intent for slot {}",
                        entity_id.local_slot.0
                    )
                }
                ExistingRecordTarget::Relation(relation_id) => {
                    format!(
                        "conflicting relation intent for slot {}",
                        relation_id.local_slot.0
                    )
                }
            };
            let _ = detail;
            return Err(CommitConflict::new(ConflictClass::ConflictingIntent {
                target,
            }));
        }
    }

    let mut seen_relation_creates = BTreeSet::new();
    for intent in intents {
        let mut identities = Vec::new();
        intent.collect_relation_identities(&mut identities);
        for identity in identities {
            if !seen_relation_creates.insert(identity) {
                return Err(CommitConflict::new(
                    ConflictClass::DuplicateRelationIdentity {
                        detail: "duplicate relation identity in merged plan".to_string(),
                    },
                ));
            }
        }
    }
    Ok(())
}
