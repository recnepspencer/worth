//! Closed collection projections share the registered view image and invalidation engine.
use super::super::{
    dependency::ViewDependency,
    retention::{ManagedDerivedSource, ManagedDerivedViewState, WorthQueryManagedDerivedValue},
};
use super::*;
use crate::basis::WorthQueryProductBranchLease;
use worth_query_declaration::facade::application_schema::{
    ApplicationRuleReadableCollectionAssociation, ApplicationSchemaBindingIdentity,
};
use worth_relational::facade::{
    identity::EntityId, runtime::DerivedRelationData, schema::DerivedRelationHandle,
};

pub(super) struct ClosedCount(pub(super) u64);
impl WorthQueryManagedDerivedValue for ClosedCount {
    fn retained_bytes(&self) -> usize {
        0
    }
}
impl ManagedDerivedViewRegistry {
    /// The caller supplies a newly acquired Native source under its actual
    /// operation contract. This cache can decline independently; it grants no
    /// decision or source authority and never substitutes for that acquisition.
    pub(in crate::domain_computation::primary_graph) fn retain_native_count(
        self: &Arc<Self>,
        runtime: u64,
        binding: &ApplicationSchemaBindingIdentity,
        definition: &DerivedRelationHandle,
        association: &ApplicationRuleReadableCollectionAssociation,
        root: EntityId,
        product: &WorthQueryProductBranchLease,
        data: &DerivedRelationData,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
    ) -> Option<u64> {
        let count = u64::try_from(data.endpoints().len()).ok()?;
        let retained =
            self.native_count_state(runtime, binding, definition, association, root, product)?;
        if retained.current_commit().as_ref() != Some(product.selected_commit()) {
            execution.check_live().ok()?;
            retained.rebase_cold(product.selected_commit().clone());
        }
        if let Ok(Some(cached)) = retained.get(&root, product.selected_commit()) {
            if cached.0 == count {
                return Some(cached.0);
            }
        }
        let mut dependencies = std::collections::BTreeSet::new();
        for (entity, _) in data.entity_dependencies() {
            execution.check_live().ok()?;
            dependencies.insert(ViewDependency::Entity(*entity));
        }
        for probe in data.adjacency_dependencies() {
            execution.check_live().ok()?;
            dependencies.insert(ViewDependency::Adjacency(
                probe.anchor,
                probe.relation,
                if probe.outgoing { 0 } else { 1 },
            ));
        }
        execution.check_live().ok()?;
        retained
            .reconstruct(
                root,
                None,
                vec![(root, ClosedCount(count), dependencies)],
                std::collections::BTreeSet::from([ViewDependency::Entity(root)]),
                product.selected_commit(),
            )
            .ok()?;
        Some(count)
    }

    fn native_count_state(
        self: &Arc<Self>,
        runtime: u64,
        binding: &ApplicationSchemaBindingIdentity,
        definition: &DerivedRelationHandle,
        association: &ApplicationRuleReadableCollectionAssociation,
        root: EntityId,
        product: &WorthQueryProductBranchLease,
    ) -> Option<Arc<ManagedDerivedViewState<EntityId, ClosedCount>>> {
        let mut state = self
            .state
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        if let Some(found) = state
            .owned
            .iter()
            .filter_map(|(_, entry)| match entry {
                OwnedDerivedView::NativeCount(state) => Some(state),
                OwnedDerivedView::ManagedIndex { .. } => None,
            })
            .find(|state| {
                state.runtime_authority == runtime
                    && &state.binding == binding
                    && state.product_branch == *product.branch_identity()
                    && state.branch == *product.relational_basis().identity().branch_id()
                    && state.incarnation == product.observation().lifecycle_incarnation()
                    && matches!(&state.source,ManagedDerivedSource::NativeCount {
                    definition:actual,association:actual_association,root:actual_root,
                } if actual==definition && actual_association==association && *actual_root==root)
            })
        {
            return Some(Arc::clone(found));
        }
        state.views.retain(|_, view| view.strong_count() > 0);
        // Both ordinary weak registrations and these framework-owned states
        // occupy this same index and its one existing capacity.
        if state.views.len() >= MAXIMUM_REGISTERED_VIEWS {
            return None;
        }
        let id = state.next_id.checked_add(1)?;
        let retained = Arc::new(ManagedDerivedViewState::native_count(
            runtime,
            binding.clone(),
            definition.clone(),
            association.clone(),
            root,
            product,
        ));
        let erased: Arc<dyn RegisteredDerivedView> = retained.clone();
        state.views.insert(id, Arc::downgrade(&erased));
        state
            .owned
            .push((id, OwnedDerivedView::NativeCount(Arc::clone(&retained))));
        state.next_id = id;
        Some(retained)
    }

    pub(in crate::domain_computation::primary_graph) fn discard_native_counts(&self) {
        loop {
            let mut state = self
                .state
                .lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner);
            let removed = state
                .owned
                .iter()
                .position(|(_, entry)| matches!(entry, OwnedDerivedView::NativeCount(_)))
                .map(|position| state.owned.remove(position));
            if let Some((id, _)) = &removed {
                state.views.remove(id);
            }
            let empty = state.take_empty_material_storage();
            drop(state);
            drop(empty);
            match removed {
                Some((_, OwnedDerivedView::NativeCount(state))) => state.dispose(),
                Some((_, OwnedDerivedView::ManagedIndex { .. })) => {
                    unreachable!("selected Native count")
                }
                None => break,
            }
        }
    }

    pub(in crate::domain_computation::primary_graph) fn native_count_lifecycle(
        &self,
    ) -> Vec<super::super::retention::WorthQueryRuleReadableCollectionLifecycleForTest> {
        let owned = self
            .state
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .owned
            .iter()
            .filter_map(|(_, entry)| match entry {
                OwnedDerivedView::NativeCount(state) => Some(Arc::clone(state)),
                OwnedDerivedView::ManagedIndex { .. } => None,
            })
            .collect::<Vec<_>>();
        owned.iter().map(|state| state.lifecycle()).collect()
    }
}
