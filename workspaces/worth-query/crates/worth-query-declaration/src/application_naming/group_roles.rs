//! Group kinds and their member roles declared as Rust types.
//!
//! A group kind is a type implementing [`ApplicationGroupKind`]; its member
//! roles are a [`WorthQueryNameRole`] enum. A typed group scheme takes its
//! group name from the kind and its role order, anchor role and boundary role
//! from role values, so no role name is ever written as text.

use super::{
    ApplicationGroupContinuityRule, ApplicationGroupNameSchemeBuilder, WorthQueryNameRole,
};

/// A declared group kind: the type a group's `WorthQueryName<Self>` is named
/// by, and the group name its installed scheme governs.
pub trait ApplicationGroupKind: 'static {
    /// The group name the kind's installed scheme governs.
    const NAME: &'static str;
}

impl<S, G: ApplicationGroupKind> ApplicationGroupNameSchemeBuilder<S, G> {
    /// Starts a group scheme for kind `G` following `rule`, whose member
    /// role order is that of `R`.
    pub fn from_roles<R: WorthQueryNameRole>(
        scheme_id: impl Into<String>,
        rule: ApplicationGroupContinuityRule,
    ) -> Self {
        Self::new(scheme_id, G::NAME, rule).role_order(R::ROLE_ORDER)
    }
}

impl<S, G> ApplicationGroupNameSchemeBuilder<S, G> {
    /// Declares the typed role whose least-named member anchors a newborn
    /// group (the producer, under `ProducerRole`).
    pub fn anchor<R: WorthQueryNameRole>(self, role: R) -> Self {
        self.anchor_role(role.role_name())
    }

    /// Declares the typed enclosing role `Boundary` continuity reads.
    pub fn boundary<R: WorthQueryNameRole>(self, role: R) -> Self {
        self.boundary_role(role.role_name())
    }
}
