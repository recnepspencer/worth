//! Atomic reconciliation of a newly certified member set with retained entries.

use std::collections::{BTreeMap, BTreeSet};
use std::sync::Arc;

use worth_relational::facade::identity::EntityId;
use worth_runtime_world::facade::CompositeCommitIdentity;

use super::{
    token_charge, DependencyIndex, ManagedDerivedViewState, RetainedEntry, RetainedMemberToken,
    ViewDependency, WorthQueryManagedDerivedMemberToken, WorthQueryManagedDerivedValue,
    WorthQueryManagedDerivedViewDenial as Denial,
};
use crate::domain_computation::primary_graph::application_query::derived_view::WorthQueryManagedDerivedViewKey;

pub(in crate::domain_computation::primary_graph::application_query::derived_view) struct MembershipReconciliationPlan
{
    revision: u64,
    commit: CompositeCommitIdentity,
    membership_key: WorthQueryManagedDerivedViewKey,
    tokens: BTreeMap<EntityId, RetainedMemberToken>,
    members: BTreeSet<EntityId>,
    required: BTreeSet<EntityId>,
}

impl MembershipReconciliationPlan {
    pub(in crate::domain_computation::primary_graph::application_query::derived_view) fn needs_read(
        &self,
        root: EntityId,
    ) -> bool {
        self.required.contains(&root)
    }

    pub(in crate::domain_computation::primary_graph::application_query::derived_view) fn read_count(
        &self,
    ) -> usize {
        self.required.len()
    }

    pub(in crate::domain_computation::primary_graph::application_query::derived_view) fn retained_count(
        &self,
    ) -> usize {
        self.members.len() - self.required.len()
    }
}

impl<Value> ManagedDerivedViewState<WorthQueryManagedDerivedViewKey, Value>
where
    Value: WorthQueryManagedDerivedValue,
{
    pub(in crate::domain_computation::primary_graph::application_query::derived_view) fn plan_membership_reconciliation<
        Member: WorthQueryManagedDerivedMemberToken,
    >(
        &self,
        membership_key: &WorthQueryManagedDerivedViewKey,
        members: &BTreeMap<EntityId, Member>,
        commit: &CompositeCommitIdentity,
    ) -> Result<MembershipReconciliationPlan, Denial> {
        let retained = self
            .retained
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        if retained.disposed {
            return Err(Denial::Disposed);
        }
        if retained.revision_exhausted {
            return Err(Denial::ViewRevisionExhausted);
        }
        if retained.cold {
            return Err(Denial::ColdReconstructionRequired);
        }
        if &retained.current_commit != commit {
            return Err(Denial::StaleSource);
        }
        if retained.membership_key.as_ref() != Some(membership_key) {
            return Err(Denial::ForeignQuery);
        }
        if !retained.membership_dirty {
            return Err(Denial::MembershipReconciliationRequired);
        }
        if members.len() > self.limits.maximum_entries() {
            return Err(Denial::EntryCapacityExceeded);
        }
        let old_tokens = retained
            .member_tokens
            .as_ref()
            .ok_or(Denial::ColdReconstructionRequired)?;
        if old_tokens.len() != retained.entries.len() {
            return Err(Denial::IncompleteDependencies);
        }
        let mut known = BTreeSet::new();
        for key in retained.entries.keys() {
            if !known.insert(key.root()) {
                return Err(Denial::IncompleteDependencies);
            }
        }
        let dirty_roots = retained
            .dirty
            .iter()
            .map(|key| key.root())
            .collect::<BTreeSet<_>>();
        let required = members
            .iter()
            .filter(|(root, token)| {
                !known.contains(root)
                    || dirty_roots.contains(root)
                    || !old_tokens.get(root).is_some_and(|old| old.matches(*token))
            })
            .map(|(root, _)| *root)
            .collect();
        let tokens = members
            .iter()
            .map(|(root, token)| (*root, RetainedMemberToken::new(token.clone())))
            .collect::<BTreeMap<_, _>>();
        if token_charge(&tokens) > self.limits.maximum_retained_bytes() {
            return Err(Denial::RetainedBytesExceeded);
        }
        Ok(MembershipReconciliationPlan {
            revision: retained.revision,
            commit: commit.clone(),
            membership_key: membership_key.clone(),
            members: members.keys().copied().collect(),
            tokens,
            required,
        })
    }

    pub(in crate::domain_computation::primary_graph::application_query::derived_view) fn reconcile_membership(
        &self,
        plan: MembershipReconciliationPlan,
        membership: BTreeSet<ViewDependency>,
        entries: Vec<(
            WorthQueryManagedDerivedViewKey,
            Value,
            BTreeSet<ViewDependency>,
        )>,
        commit: &CompositeCommitIdentity,
    ) -> Result<(Vec<WorthQueryManagedDerivedViewKey>, usize), Denial> {
        let mut retained = self
            .retained
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        if retained.disposed {
            return Err(Denial::Disposed);
        }
        if retained.revision_exhausted || retained.revision == u64::MAX {
            retained.revision_exhausted = true;
            retained.cold = true;
            return Err(Denial::ViewRevisionExhausted);
        }
        if retained.cold {
            return Err(Denial::ColdReconstructionRequired);
        }
        if &retained.current_commit != commit
            || plan.commit != *commit
            || retained.revision != plan.revision
        {
            return Err(Denial::StaleSource);
        }
        if !retained.membership_dirty
            || retained.membership_key.as_ref() != Some(&plan.membership_key)
            || membership.is_empty()
        {
            return Err(Denial::IncompleteDependencies);
        }
        let mut supplied = BTreeSet::new();
        let mut incoming_charge = 0usize;
        for (key, value, dependencies) in &entries {
            if !plan.required.contains(&key.root())
                || !supplied.insert(key.root())
                || dependencies.is_empty()
            {
                return Err(Denial::IncompleteDependencies);
            }
            incoming_charge =
                incoming_charge.saturating_add(entry_charge::<Value>(value, dependencies));
        }
        if supplied != plan.required {
            return Err(Denial::IncompleteDependencies);
        }
        let removed = retained
            .entries
            .iter()
            .filter(|(key, _)| {
                !plan.members.contains(&key.root()) || plan.required.contains(&key.root())
            })
            .map(|(key, entry)| (key.clone(), retained_entry_charge(entry)))
            .collect::<Vec<_>>();
        let removed_members = retained
            .entries
            .keys()
            .filter(|key| !plan.members.contains(&key.root()))
            .count();
        let outgoing_charge = removed
            .iter()
            .fold(0usize, |bytes, (_, charge)| bytes.saturating_add(*charge));
        let old_dirty_reserve = dirty_reserve(retained.entries.len());
        let new_dirty_reserve = dirty_reserve(plan.members.len());
        let old_token_charge = retained.member_tokens.as_ref().map_or(0, token_charge);
        let required_bytes = retained
            .charged_bytes
            .checked_sub(membership_charge::<WorthQueryManagedDerivedViewKey>(
                &retained.membership,
            ))
            .and_then(|bytes| bytes.checked_sub(outgoing_charge))
            .and_then(|bytes| bytes.checked_sub(old_dirty_reserve))
            .and_then(|bytes| bytes.checked_sub(old_token_charge))
            .ok_or(Denial::IncompleteDependencies)?
            .saturating_add(membership_charge::<WorthQueryManagedDerivedViewKey>(
                &membership,
            ))
            .saturating_add(incoming_charge)
            .saturating_add(new_dirty_reserve)
            .saturating_add(token_charge(&plan.tokens));
        if required_bytes > self.limits.maximum_retained_bytes() {
            return Err(Denial::RetainedBytesExceeded);
        }
        let old_membership = std::mem::take(&mut retained.membership);
        retained.index.remove_membership(&old_membership);
        for (key, _) in removed {
            let old = retained
                .entries
                .remove(&key)
                .expect("planned retained entry");
            retained.index.remove_entry(&key, &old.dependencies);
        }
        for dependency in &membership {
            retained.index.insert_membership(dependency);
        }
        retained.membership = membership;
        retained.member_tokens = Some(plan.tokens);
        for (key, value, dependencies) in entries {
            for dependency in &dependencies {
                retained.index.insert_entry(dependency, &key);
            }
            retained.entries.insert(
                key,
                RetainedEntry {
                    value: Arc::new(value),
                    dependencies,
                },
            );
        }
        retained.dirty.clear();
        retained.membership_dirty = false;
        retained.charged_bytes = required_bytes;
        retained.advance_revision();
        Ok((retained.entries.keys().cloned().collect(), removed_members))
    }
}

fn membership_charge<Key: Clone + Ord>(dependencies: &BTreeSet<ViewDependency>) -> usize {
    dependencies.iter().fold(0usize, |bytes, dependency| {
        bytes
            .saturating_add(dependency.retained_bytes())
            .saturating_add(DependencyIndex::<Key>::entry_insertion_bound(dependency))
    })
}

fn entry_charge<Value: WorthQueryManagedDerivedValue>(
    value: &Value,
    dependencies: &BTreeSet<ViewDependency>,
) -> usize {
    value
        .retained_bytes()
        .saturating_add(std::mem::size_of::<(
            WorthQueryManagedDerivedViewKey,
            RetainedEntry<Value>,
        )>())
        .saturating_add(4 * std::mem::size_of::<usize>())
        .saturating_add(membership_charge::<WorthQueryManagedDerivedViewKey>(
            dependencies,
        ))
}

fn retained_entry_charge<Value: WorthQueryManagedDerivedValue>(
    entry: &RetainedEntry<Value>,
) -> usize {
    entry_charge(entry.value.as_ref(), &entry.dependencies)
}

fn dirty_reserve(entries: usize) -> usize {
    entries.saturating_mul(
        std::mem::size_of::<WorthQueryManagedDerivedViewKey>() + 4 * std::mem::size_of::<usize>(),
    )
}
