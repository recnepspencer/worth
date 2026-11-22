//! Fold the authored lineage edges; posting order never chooses an alternative.
use super::*;
use crate::domain_computation::application_aftermath::WorthQueryAftermathCausalRole as Role;
use std::collections::BTreeSet;
pub(super) fn readmit(
    facts: &BTreeMap<CommitId, WorthQueryCanonicalCorrectionFact>,
) -> Result<(), Denial> {
    let mut active = BTreeMap::new();
    let mut reversed = BTreeSet::new();
    for fact in facts.values() {
        let deny = || Denial::InconsistentRelationships {
            record: fact.record.clone(),
        };
        if facts.contains_key(&fact.root.relational().commit_id) {
            return Err(deny());
        }
        match fact.role {
            Role::RecordedReverse => {
                match &fact.prior {
                    None => {
                        if fact.source != fact.root {
                            return Err(deny());
                        }
                    }
                    Some(prior) => {
                        let previous = facts.get(&prior.relational().commit_id).ok_or_else(deny)?;
                        if prior != &fact.source
                            || previous.role != Role::RecordedReapply
                            || previous.root != fact.root
                        {
                            return Err(deny());
                        }
                    }
                }
                if !reversed.insert(fact.source.relational().commit_id)
                    || active
                        .insert(fact.root.relational().commit_id, fact.child.commit_id)
                        .is_some()
                {
                    return Err(deny());
                }
            }
            Role::RecordedReapply => {
                let selected = fact.selected.as_ref().ok_or_else(deny)?;
                let consumed = fact.active.as_ref().ok_or_else(deny)?;
                let selected_fact = facts
                    .get(&selected.relational().commit_id)
                    .ok_or_else(deny)?;
                let consumed_fact = facts
                    .get(&consumed.relational().commit_id)
                    .ok_or_else(deny)?;
                if selected_fact.role != Role::RecordedReverse
                    || consumed_fact.role != Role::RecordedReverse
                    || selected_fact.root != fact.root
                    || consumed_fact.root != fact.root
                    || selected_fact.source != fact.source
                    || fact.prior.as_ref() != Some(consumed)
                    || active.remove(&fact.root.relational().commit_id)
                        != Some(consumed_fact.child.commit_id)
                {
                    return Err(deny());
                }
            }
        }
    }
    Ok(())
}
