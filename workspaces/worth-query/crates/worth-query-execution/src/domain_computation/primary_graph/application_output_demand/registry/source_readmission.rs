//! Bounded custody of the original query input for framework readmission.

use std::any::Any;
use std::ops::Deref;
use std::sync::Arc;

use super::{
    required_custody::RequiredOutputCustodyCapacity, WorthQueryOutputDemandInterest,
    WorthQueryOutputDemandRegistry,
};
use crate::domain_computation::execution_runtime::WorthQueryOutputDemandLimits;
use crate::domain_computation::primary_graph::output_lineage::invalidation::InvalidationEditAdmission;
use crate::domain_computation::primary_graph::{
    WorthQueryObservedSource, WorthQueryOutputDemandDenial, WorthQueryOutputDemandDenialKind,
};

/// The observation is moved once. Its existing selector, parameter and source
/// capacity tickets remain inside it; neither observer duplicates that backing.
pub(in crate::domain_computation::primary_graph) struct RetainedOutputReadmissionSource<Query> {
    source: WorthQueryObservedSource<Query>,
    _capacity: RequiredOutputCustodyCapacity,
}

impl<Query> Deref for RetainedOutputReadmissionSource<Query> {
    type Target = WorthQueryObservedSource<Query>;

    fn deref(&self) -> &Self::Target {
        &self.source
    }
}

impl<Query> RetainedOutputReadmissionSource<Query> {
    pub(in crate::domain_computation::primary_graph) fn source(
        &self,
    ) -> &WorthQueryObservedSource<Query> {
        &self.source
    }
}

/// Erasure transports an owner-issued source to its installed typed executor.
/// It does not authorize a read, a branch selection, or producer execution.
pub(in crate::domain_computation::primary_graph) struct RequiredOutputReadmission {
    pub(in crate::domain_computation::primary_graph) source: Arc<dyn Any + Send + Sync>,
    pub(in crate::domain_computation::primary_graph) limits: WorthQueryOutputDemandLimits,
    pub(in crate::domain_computation::primary_graph) retained_program_basis:
        Option<Arc<crate::domain_computation::primary_graph::WorthQueryApplicationReadObservation>>,
}

pub(in crate::domain_computation::primary_graph) struct PreparedOutputReadmissionSource<Query> {
    owner: WorthQueryOutputDemandRegistry,
    source: Arc<RetainedOutputReadmissionSource<Query>>,
    readmission: Arc<RequiredOutputReadmission>,
}

impl<Query> Deref for PreparedOutputReadmissionSource<Query> {
    type Target = WorthQueryObservedSource<Query>;

    fn deref(&self) -> &Self::Target {
        &self.source
    }
}

impl WorthQueryOutputDemandRegistry {
    pub(in crate::domain_computation::primary_graph) fn retain_readmission_source<
        Query: 'static,
    >(
        &self,
        source: WorthQueryObservedSource<Query>,
        limits: WorthQueryOutputDemandLimits,
        retained_program_basis: Option<
            Arc<crate::domain_computation::primary_graph::WorthQueryApplicationReadObservation>,
        >,
        admission: &mut InvalidationEditAdmission,
    ) -> Result<PreparedOutputReadmissionSource<Query>, WorthQueryOutputDemandDenial> {
        let bytes = std::mem::size_of::<RetainedOutputReadmissionSource<Query>>()
            .checked_add(std::mem::size_of::<RequiredOutputReadmission>())
            .and_then(|bytes| bytes.checked_add(4 * std::mem::size_of::<usize>()))
            .ok_or_else(capacity_denial)?;
        // This wrapper is registry custody. The producer's retained-output
        // limit governs its output, while the shared required registry limit
        // governs this readmission source. A cached Ready nothing holds
        // gives way to it.
        self.reclaim_cached_rows(bytes, admission)?;
        let capacity = self.reserve_required_custody_capacity(bytes)?;
        let source = Arc::new(RetainedOutputReadmissionSource {
            source,
            _capacity: capacity,
        });
        let erased = Arc::clone(&source) as Arc<dyn Any + Send + Sync>;
        Ok(PreparedOutputReadmissionSource {
            owner: self.clone(),
            source,
            readmission: Arc::new(RequiredOutputReadmission {
                source: erased,
                limits,
                retained_program_basis,
            }),
        })
    }
}

impl<Query: 'static> PreparedOutputReadmissionSource<Query> {
    pub(in crate::domain_computation::primary_graph) fn install(
        self,
        interest: &WorthQueryOutputDemandInterest,
    ) -> Result<Arc<RetainedOutputReadmissionSource<Query>>, WorthQueryOutputDemandDenial> {
        if !Arc::ptr_eq(&self.owner.state, &interest.owner.state) {
            return Err(source_denial());
        }
        let epoch = self
            .source
            .output_source_epoch()
            .ok_or_else(source_denial)?;
        let occurrence = self
            .source
            .selected_product_occurrence()
            .ok_or_else(source_denial)?;
        let mut state = self
            .owner
            .state
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        let record = state
            .records
            .get_mut(&interest.key)
            .ok_or_else(source_denial)?;
        if !interest.key.source.same_semantic_source(&epoch)
            || record.product_occurrence != occurrence
            || record.source_scope != Some(
                crate::domain_computation::authorization::WorthQueryOperationScopeEntityBinding::from_entity(self.source.source_root())
            )
        {
            return Err(source_denial());
        }
        if record.readmission_source.is_none() {
            record.readmission_source = Some(self.readmission);
        }
        drop(state);
        Ok(self.source)
    }
}
fn capacity_denial() -> WorthQueryOutputDemandDenial {
    WorthQueryOutputDemandDenial::new(
        WorthQueryOutputDemandDenialKind::RetentionBudgetExceeded,
        "required output source readmission exceeds retained capacity",
    )
}

fn source_denial() -> WorthQueryOutputDemandDenial {
    WorthQueryOutputDemandDenial::new(
        WorthQueryOutputDemandDenialKind::ForeignSource,
        "required output readmission source does not match its admitted record",
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::Ordering;

    #[test]
    fn readmission_and_index_share_one_limit_and_final_owner_refunds_capacity() {
        let registry = WorthQueryOutputDemandRegistry::with_budgets(64, 64, 64);
        registry.state.lock().unwrap().required_reserved_bytes = 40;
        let first = Arc::new(registry.reserve_required_custody_capacity(16).unwrap());
        let observer = Arc::clone(&first);
        assert!(registry.reserve_required_custody_capacity(9).is_err());
        assert!(!registry.state.lock().unwrap().has_required_capacity(49));

        drop(first);
        assert!(registry.reserve_required_custody_capacity(9).is_err());
        drop(observer);
        let replacement = registry.reserve_required_custody_capacity(24).unwrap();
        assert!(registry.reserve_required_custody_capacity(1).is_err());
        drop(replacement);
        assert!(registry.state.lock().unwrap().has_required_capacity(64));
        assert!(registry.reserve_required_custody_capacity(65).is_err());
        assert_eq!(
            registry
                .state
                .lock()
                .unwrap()
                .required_custody_retained_bytes
                .load(Ordering::Acquire),
            0
        );
    }
}
