use std::collections::BTreeSet;
use std::sync::Arc;

use worth_runtime_world::facade::CompositeCommitIdentity;

use super::{
    verify_observation, ManagedDerivedViewState, ViewDependency, WorthQueryManagedDerivedValue,
    WorthQueryManagedDerivedViewDenial,
};

impl<Key, Value> ManagedDerivedViewState<Key, Value>
where
    Key: Clone + Ord,
    Value: WorthQueryManagedDerivedValue,
{
    pub(in crate::domain_computation::primary_graph::application_query::derived_view) fn requires_entry_refresh(
        &self,
        key: &Key,
        commit: &CompositeCommitIdentity,
    ) -> Result<(), WorthQueryManagedDerivedViewDenial> {
        let retained = self
            .retained
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        verify_observation(&retained, commit)?;
        if retained.dirty.contains(key) {
            Ok(())
        } else {
            Err(WorthQueryManagedDerivedViewDenial::EntryRefreshRequired)
        }
    }

    pub(in crate::domain_computation::primary_graph::application_query::derived_view) fn refresh_entry(
        &self,
        key: &Key,
        value: Value,
        dependencies: BTreeSet<ViewDependency>,
        commit: &CompositeCommitIdentity,
    ) -> Result<Arc<Value>, WorthQueryManagedDerivedViewDenial> {
        let mut retained = self
            .retained
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        verify_observation(&retained, commit)?;
        if !retained.dirty.contains(key) {
            return Err(WorthQueryManagedDerivedViewDenial::EntryRefreshRequired);
        }
        if retained.revision == u64::MAX {
            retained.revision_exhausted = true;
            retained.cold = true;
            return Err(WorthQueryManagedDerivedViewDenial::ViewRevisionExhausted);
        }
        if dependencies.is_empty() {
            return Err(WorthQueryManagedDerivedViewDenial::IncompleteDependencies);
        }
        let Some(old) = retained.entries.get(key) else {
            return Err(WorthQueryManagedDerivedViewDenial::IncompleteDependencies);
        };
        let old_charge =
            super::storage_quote::entry_quote::<Key, Value>(old.value.as_ref(), &old.dependencies)?;
        let new_charge = super::storage_quote::entry_quote::<Key, Value>(&value, &dependencies)?;
        let required = retained
            .charged_bytes
            .checked_sub(old_charge)
            .ok_or(WorthQueryManagedDerivedViewDenial::IncompleteDependencies)?
            .checked_add(new_charge)
            .ok_or(WorthQueryManagedDerivedViewDenial::RetainedBytesExceeded)?;
        if self.limits.rejects_bytes(required) {
            return Err(WorthQueryManagedDerivedViewDenial::RetainedBytesExceeded);
        }
        let old_dependencies = old.dependencies.clone();
        retained.index.remove_entry(key, &old_dependencies);
        for dependency in &dependencies {
            retained.index.insert_entry(dependency, key);
        }
        let entry = retained
            .entries
            .get_mut(key)
            .expect("checked retained entry");
        entry.dependencies = dependencies;
        let value = Arc::new(value);
        entry.value = Arc::clone(&value);
        retained.charged_bytes = required;
        retained.dirty.remove(key);
        retained.advance_revision();
        Ok(value)
    }
}
