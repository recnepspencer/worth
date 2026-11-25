//! A Home program transition commits its target steps through the predecessor's
//! active companion. While the transition holds this attachment, a preparation
//! Native runs under the successor's exact Home catalog resolves its Source
//! programs from the successor's compiled installation; Source images stay in
//! the active companion's own cells and registration generation.
use super::*;
use crate::domain_computation::primary_graph::application_query::managed_index::{
    source::service::SourceService, ManagedSourceInstallation,
};
use worth_relational::facade::runtime::RelationalHomeProgramCatalogReceipt;

pub struct TransitionTargetSources {
    sources: Arc<ManagedSourceInstallation>,
    catalog: RelationalHomeProgramCatalogReceipt,
}

/// Held by the transition across its positioned steps, including refusal.
pub struct TransitionTargetSourcesGuard {
    owner: Arc<ManagedDerivedViewRegistry>,
    target: Arc<TransitionTargetSources>,
}

impl ManagedDerivedViewRegistry {
    /// `None` when another transition already holds this registry's target.
    pub fn attach_transition_target_sources(
        self: &Arc<Self>,
        sources: Arc<ManagedSourceInstallation>,
        catalog: RelationalHomeProgramCatalogReceipt,
    ) -> Option<TransitionTargetSourcesGuard> {
        let mut active = self
            .transition_target
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        if active.strong_count() != 0 {
            return None;
        }
        let target = Arc::new(TransitionTargetSources { sources, catalog });
        *active = Arc::downgrade(&target);
        drop(active);
        Some(TransitionTargetSourcesGuard {
            owner: Arc::clone(self),
            target,
        })
    }

    /// The successor's service answers only preparations under its own catalog.
    pub(super) fn source_service(
        &self,
        source: &ManagedIndexSourcePreparation<'_>,
    ) -> Result<Arc<dyn SourceService>, Denial> {
        let target = self
            .transition_target
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .upgrade();
        let successor = target
            .as_ref()
            .filter(|target| {
                source
                    .home_catalog()
                    .is_some_and(|catalog| catalog.same_inventory(&target.catalog))
            })
            .and_then(|target| target.sources.service(source.program()));
        successor
            .or_else(|| self.sources.service(source.program()))
            .map(Arc::clone)
            .ok_or(Denial::MissingParticipant)
    }
}

impl Drop for TransitionTargetSourcesGuard {
    fn drop(&mut self) {
        let mut active = self
            .owner
            .transition_target
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        if Weak::ptr_eq(&active, &Arc::downgrade(&self.target)) {
            *active = Weak::new();
        }
        drop(active);
    }
}
