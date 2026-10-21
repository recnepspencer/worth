//! Bounded custody of the original query input for framework readmission.

use std::any::Any;
use std::ops::Deref;
use std::sync::Arc;
mod cached_successor;

use super::WorthQueryOutputDemandInterest;
use super::WorthQueryOutputDemandRegistry;
use crate::domain_computation::execution_runtime::WorthQueryOutputDemandLimits;

use crate::domain_computation::primary_graph::WorthQueryObservedSource;
use crate::domain_computation::primary_graph::WorthQueryOutputDemandDenial;
use crate::domain_computation::primary_graph::WorthQueryOutputDemandDenialKind;

/// The observation is moved once. Its existing selector, parameter and source
/// capacity tickets remain inside it; neither observer duplicates that backing.
pub(in crate::domain_computation::primary_graph) struct RetainedOutputReadmissionSource<Query> {
    source: WorthQueryObservedSource<Query>,
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
    /// Transport the immutable installed entry selected for this exact row.
    /// The typed demand validates its family and identity after downcasting.
    pub(in crate::domain_computation::primary_graph) producer:
        std::sync::OnceLock<Arc<dyn Any + Send + Sync>>,
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
    ) -> Result<PreparedOutputReadmissionSource<Query>, WorthQueryOutputDemandDenial> {
        // This wrapper is registry custody. The producer's retained-output
        // limit governs its output, while the shared required registry limit
        // governs this readmission source. A cached Ready nothing holds
        // gives way to it.

        let source = Arc::new(RetainedOutputReadmissionSource { source });
        let erased = Arc::clone(&source) as Arc<dyn Any + Send + Sync>;
        Ok(PreparedOutputReadmissionSource {
            owner: self.clone(),
            source,
            readmission: Arc::new(RequiredOutputReadmission {
                source: erased,
                producer: std::sync::OnceLock::new(),
                limits,
                retained_program_basis,
            }),
        })
    }
}

impl<Query: 'static> PreparedOutputReadmissionSource<Query> {
    /// The retained source of a demand that holds no registry row, so no
    /// row readmits it.
    pub(in crate::domain_computation::primary_graph) fn without_row(
        self,
    ) -> Arc<RetainedOutputReadmissionSource<Query>> {
        self.source
    }

    pub(in crate::domain_computation::primary_graph) fn install(
        self,
        interest: &WorthQueryOutputDemandInterest,
        producer: Arc<dyn Any + Send + Sync>,
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
        let mut record = state
            .records
            .get_mut(&interest.key)
            .ok_or_else(source_denial)?;
        if record.product_occurrence != occurrence
            || record.source_scope != Some(
                crate::domain_computation::authorization::WorthQueryOperationScopeEntityBinding::from_entity(self.source.source_root())
            )
        {
            return Err(source_denial());
        }
        if !interest.key.source.same_semantic_source(&epoch) {
            // The caller retains its genuinely fresh source alongside the
            // predecessor. Never install it under the predecessor's old key.
            // Its original Ready/source/member pins still enter the ordinary
            // selected certification and accepted-authority refresh path.
            cached_successor::validate_original_ready(&record, &interest.key, &epoch, &producer)?;
            return Ok(self.source);
        }
        // Readmission retains frozen source meaning. Each read selects the
        // current Product; a semantic join does not need to replace this Arc.
        if record.readmission_source.is_none() {
            self.readmission
                .producer
                .set(producer)
                .unwrap_or_else(|_| unreachable!("prepared readmission is installed once"));
            record.readmission_source = Some(self.readmission);
        }
        record.finish();
        drop(state);
        Ok(self.source)
    }
}

fn source_denial() -> WorthQueryOutputDemandDenial {
    WorthQueryOutputDemandDenial::new(
        WorthQueryOutputDemandDenialKind::ForeignSource,
        "required output readmission source does not match its admitted record",
    )
}
