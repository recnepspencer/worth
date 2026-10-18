use std::collections::BTreeSet;

use crate::{
    WorthUiInstalledQueryBindingReference, WorthUiOperationLiveRetirement, WorthUiQueryViewIdentity,
};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum WorthUiQueryBindingSuccessionDenial {
    DuplicateSuccessorView,
    EmptyChange,
    ForeignSuccessorReference,
    StaleSuccessorReference,
    UnpublishedLiveChanges,
}

#[derive(Clone, Debug)]
pub struct WorthUiQueryBindingSuccessionChange {
    predecessor: Option<WorthUiInstalledQueryBindingReference>,
    successor: Option<WorthUiInstalledQueryBindingReference>,
}

impl WorthUiQueryBindingSuccessionChange {
    pub fn new(
        predecessor: Option<WorthUiInstalledQueryBindingReference>,
        successor: Option<WorthUiInstalledQueryBindingReference>,
    ) -> Self {
        Self {
            predecessor,
            successor,
        }
    }
}

#[must_use = "prepared Query succession must be committed or abandoned with its candidate resources"]
pub struct WorthUiPreparedQueryBindingSuccession {
    candidate: super::WorthUiRuntimeQueryBinding,
    scope: WorthUiQueryBindingSuccessionScope,
    retirement: WorthUiOperationLiveRetirement,
}

enum WorthUiQueryBindingSuccessionScope {
    Complete {
        carry_references: Vec<WorthUiInstalledQueryBindingReference>,
        retained: BTreeSet<WorthUiQueryViewIdentity>,
    },
    Regional {
        carry_references: Vec<WorthUiInstalledQueryBindingReference>,
    },
}

impl super::WorthUiRuntimeQueryBinding {
    pub fn prepare_succession(
        self,
        active: &Self,
        successor_references: impl IntoIterator<Item = WorthUiInstalledQueryBindingReference>,
    ) -> Result<WorthUiPreparedQueryBindingSuccession, WorthUiQueryBindingSuccessionDenial> {
        if !self.installation_is_current() {
            return Err(WorthUiQueryBindingSuccessionDenial::StaleSuccessorReference);
        }
        deny_unpublished_live_changes(active, &self)?;
        let successor_references = successor_references.into_iter().collect::<Vec<_>>();
        let mut identities = BTreeSet::<WorthUiQueryViewIdentity>::new();
        for reference in &successor_references {
            if !identities.insert(reference.definition().identity().clone()) {
                return Err(WorthUiQueryBindingSuccessionDenial::DuplicateSuccessorView);
            }
            if !self.admits_reference(reference) {
                return Err(WorthUiQueryBindingSuccessionDenial::ForeignSuccessorReference);
            }
            if !reference.installation_is_current() {
                return Err(WorthUiQueryBindingSuccessionDenial::StaleSuccessorReference);
            }
        }
        let carry_references = successor_references
            .iter()
            .filter(|reference| active.admits_reference(reference))
            .cloned()
            .collect();
        Ok(WorthUiPreparedQueryBindingSuccession {
            retirement: prepared_retirement(active, &self),
            candidate: self,
            scope: WorthUiQueryBindingSuccessionScope::Complete {
                carry_references,
                retained: identities,
            },
        })
    }

    pub fn prepare_regional_succession(
        self,
        active: &Self,
        changes: impl IntoIterator<Item = WorthUiQueryBindingSuccessionChange>,
    ) -> Result<WorthUiPreparedQueryBindingSuccession, WorthUiQueryBindingSuccessionDenial> {
        if !self.installation_is_current() {
            return Err(WorthUiQueryBindingSuccessionDenial::StaleSuccessorReference);
        }
        deny_unpublished_live_changes(active, &self)?;
        let changes = changes.into_iter().collect::<Vec<_>>();
        let mut affected = BTreeSet::<WorthUiQueryViewIdentity>::new();
        let mut preserved = BTreeSet::<WorthUiQueryViewIdentity>::new();
        for change in &changes {
            if change
                .predecessor
                .as_ref()
                .is_some_and(|reference| !active.admits_reference(reference))
            {
                return Err(WorthUiQueryBindingSuccessionDenial::ForeignSuccessorReference);
            }
            if change
                .successor
                .as_ref()
                .is_some_and(|reference| !self.admits_reference(reference))
            {
                return Err(WorthUiQueryBindingSuccessionDenial::ForeignSuccessorReference);
            }
            if change
                .successor
                .as_ref()
                .is_some_and(|reference| !reference.installation_is_current())
            {
                return Err(WorthUiQueryBindingSuccessionDenial::StaleSuccessorReference);
            }
            register_affected_identities(change, &mut affected)?;
            if change.predecessor == change.successor {
                preserved.insert(
                    change
                        .successor
                        .as_ref()
                        .expect("equal regional Query change carries both sides")
                        .definition()
                        .identity()
                        .clone(),
                );
            }
        }
        let carry_references = self
            .installed_references()
            .into_iter()
            .filter(|reference| {
                active.admits_reference(reference)
                    && (!affected.contains(reference.definition().identity())
                        || preserved.contains(reference.definition().identity()))
            })
            .collect();
        Ok(WorthUiPreparedQueryBindingSuccession {
            retirement: prepared_retirement(active, &self),
            candidate: self,
            scope: WorthUiQueryBindingSuccessionScope::Regional { carry_references },
        })
    }
}

impl WorthUiPreparedQueryBindingSuccession {
    pub fn candidate(&self) -> &super::WorthUiRuntimeQueryBinding {
        &self.candidate
    }

    pub fn commit_once(
        self,
        active: &mut super::WorthUiRuntimeQueryBinding,
    ) -> WorthUiOperationLiveRetirement {
        let mut predecessor =
            std::mem::replace(active, super::WorthUiRuntimeQueryBinding::QueryFree);
        let mut successor = self.candidate;
        let mut retirement = self.retirement;

        match self.scope {
            WorthUiQueryBindingSuccessionScope::Complete {
                carry_references,
                retained,
            } => {
                commit_prepared_carries(
                    &mut predecessor,
                    &mut successor,
                    &carry_references,
                    &mut retirement,
                );
                successor.retain_only_operation_live_resources_for(&retained, &mut retirement);
                successor.retain_only_settlements_for(&retained);
            }
            WorthUiQueryBindingSuccessionScope::Regional { carry_references } => {
                commit_prepared_carries(
                    &mut predecessor,
                    &mut successor,
                    &carry_references,
                    &mut retirement,
                );
            }
        }
        predecessor.drain_operation_live_resources_into(&mut retirement);
        successor.finish_operation_live_succession(&mut retirement);
        *active = successor;
        retirement
    }
}

fn deny_unpublished_live_changes(
    active: &super::WorthUiRuntimeQueryBinding,
    candidate: &super::WorthUiRuntimeQueryBinding,
) -> Result<(), WorthUiQueryBindingSuccessionDenial> {
    if active.has_staged_operation_live_changes() || candidate.has_staged_operation_live_changes() {
        return Err(WorthUiQueryBindingSuccessionDenial::UnpublishedLiveChanges);
    }
    Ok(())
}

fn register_affected_identities(
    change: &WorthUiQueryBindingSuccessionChange,
    affected: &mut BTreeSet<WorthUiQueryViewIdentity>,
) -> Result<(), WorthUiQueryBindingSuccessionDenial> {
    let predecessor = change
        .predecessor
        .as_ref()
        .map(|reference| reference.definition().identity());
    let successor = change
        .successor
        .as_ref()
        .map(|reference| reference.definition().identity());
    if predecessor.is_none() && successor.is_none() {
        return Err(WorthUiQueryBindingSuccessionDenial::EmptyChange);
    }
    for identity in [predecessor, successor].into_iter().flatten() {
        if predecessor == successor && predecessor == Some(identity) {
            if affected.contains(identity) {
                return Err(WorthUiQueryBindingSuccessionDenial::DuplicateSuccessorView);
            }
            affected.insert(identity.clone());
            break;
        }
        if !affected.insert(identity.clone()) {
            return Err(WorthUiQueryBindingSuccessionDenial::DuplicateSuccessorView);
        }
    }
    Ok(())
}

fn prepared_retirement(
    active: &super::WorthUiRuntimeQueryBinding,
    candidate: &super::WorthUiRuntimeQueryBinding,
) -> WorthUiOperationLiveRetirement {
    let capacity = active
        .operation_live_resource_count()
        .saturating_add(candidate.operation_live_resource_count());
    WorthUiOperationLiveRetirement::with_resource_capacity(capacity)
}

fn commit_prepared_carries(
    predecessor: &mut super::WorthUiRuntimeQueryBinding,
    successor: &mut super::WorthUiRuntimeQueryBinding,
    carry_references: &[WorthUiInstalledQueryBindingReference],
    retirement: &mut WorthUiOperationLiveRetirement,
) {
    for reference in carry_references {
        if let Some(projection) = predecessor.take_settled_snapshot(reference) {
            successor.take_settled_snapshot(reference);
            successor.replace_settled_snapshot(projection);
        }
        if let Some(resource) = predecessor.take_operation_live_resource(reference)
            && let Some(displaced) = successor.replace_operation_live_resource(reference, resource)
        {
            retirement.extend(std::iter::once(displaced));
        }
    }
}
