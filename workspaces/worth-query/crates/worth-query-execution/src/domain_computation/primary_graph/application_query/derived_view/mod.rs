mod dependency;
mod registration;
mod registry;
mod retention;

use worth_foundational::facade::CanonicalDigestId;
use worth_relational::facade::identity::EntityId;

pub(in crate::domain_computation::primary_graph) use registry::ManagedDerivedViewRegistry;
pub use retention::{WorthQueryManagedDerivedValue, WorthQueryManagedDerivedViewDenial};

/// Query-issued row identity: an exact source root and parameter binding,
/// never a consumer-selected scene key.
#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub struct WorthQueryManagedDerivedViewKey {
    model_root: EntityId,
    root: EntityId,
    parameters: CanonicalDigestId,
}

impl WorthQueryManagedDerivedViewKey {
    const fn new(model_root: EntityId, root: EntityId, parameters: CanonicalDigestId) -> Self {
        Self {
            model_root,
            root,
            parameters,
        }
    }

    pub const fn model_root(&self) -> EntityId {
        self.model_root
    }

    pub const fn root(&self) -> EntityId {
        self.root
    }

    pub const fn parameters(&self) -> &CanonicalDigestId {
        &self.parameters
    }
}

impl<Query> super::WorthQueryObservedSource<Query> {
    /// A lookup key from Query's issued row source, not a scene-authored key.
    pub fn managed_derived_view_key(&self) -> WorthQueryManagedDerivedViewKey {
        WorthQueryManagedDerivedViewKey::new(
            self.model_root,
            self.source_root(),
            self.parameter_binding_identity,
        )
    }
}

pub type WorthQueryManagedDerivedView<Query, Value> =
    retention::WorthQueryManagedDerivedView<Query, WorthQueryManagedDerivedViewKey, Value>;
pub type WorthQueryManagedDerivedViewSnapshot<Value> =
    retention::WorthQueryManagedDerivedViewSnapshot<WorthQueryManagedDerivedViewKey, Value>;
