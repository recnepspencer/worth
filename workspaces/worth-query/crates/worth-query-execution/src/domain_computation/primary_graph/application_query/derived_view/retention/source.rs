//! Purpose identity is independent of the shared retained-image lifecycle.
use super::*;
use worth_query_declaration::facade::application_schema::ApplicationRuleReadableCollectionAssociation;
use worth_relational::facade::schema::DerivedRelationHandle;

/// Read-only lifecycle of one framework-owned projection. This contains no
/// source identity, dependency keys, cached values or read authority.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum WorthQueryRuleReadableCollectionLifecycleForTest {
    Cold,
    Dirty,
    Warm,
    Disposed,
}

pub(in crate::domain_computation::primary_graph::application_query::derived_view) enum ManagedDerivedSource
{
    Query(WorthQueryInstalledApplicationQueryIdentity),
    NativeCount {
        definition: DerivedRelationHandle,
        association: ApplicationRuleReadableCollectionAssociation,
        root: EntityId,
    },
}
impl ManagedDerivedSource {
    pub(in crate::domain_computation::primary_graph::application_query::derived_view) fn query(
        &self,
    ) -> Option<&WorthQueryInstalledApplicationQueryIdentity> {
        match self {
            Self::Query(query) => Some(query),
            Self::NativeCount { .. } => None,
        }
    }
}

impl<Key: Clone + Ord, Value: WorthQueryManagedDerivedValue> ManagedDerivedViewState<Key, Value> {
    pub(in crate::domain_computation::primary_graph::application_query::derived_view) fn lifecycle(
        &self,
    ) -> WorthQueryRuleReadableCollectionLifecycleForTest {
        use WorthQueryRuleReadableCollectionLifecycleForTest as State;
        let state = self
            .retained
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        if state.disposed {
            State::Disposed
        } else if state.cold || state.revision_exhausted || state.membership_dirty {
            State::Cold
        } else if !state.dirty.is_empty() {
            State::Dirty
        } else {
            State::Warm
        }
    }
    pub(in crate::domain_computation::primary_graph::application_query::derived_view) fn native_count(
        runtime_authority: u64,
        binding: ApplicationSchemaBindingIdentity,
        definition: DerivedRelationHandle,
        association: ApplicationRuleReadableCollectionAssociation,
        root: EntityId,
        product: &crate::basis::WorthQueryProductBranchLease,
    ) -> Self {
        Self {
            runtime_authority,
            binding,
            source: ManagedDerivedSource::NativeCount {
                definition,
                association,
                root,
            },
            entry_query: None,
            secondary_entry_query: None,
            branch: product.relational_basis().identity().branch_id().clone(),
            product_branch: product.branch_identity().clone(),
            incarnation: product.observation().lifecycle_incarnation(),
            limits: ManagedStoragePolicy::OwnerSized,
            retained: Mutex::new(RetainedView {
                current_commit: product.selected_commit().clone(),
                membership_key: None,
                member_tokens: None,
                membership: BTreeSet::new(),
                entries: BTreeMap::new(),
                index: DependencyIndex::default(),
                dirty: BTreeSet::new(),
                charged_bytes: 0,
                revision: 0,
                revision_exhausted: false,
                cold: true,
                membership_dirty: false,
                disposed: false,
            }),
        }
    }
}
