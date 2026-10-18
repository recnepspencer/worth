//! Covered publication selection for an already qualified native source.
//!
//! Unlike current-entity selection this retains descriptive and retirement
//! heads, and refuses unresolved reservations. A missing head alone never
//! constructs a first-publication classification.

use crate::domain_computation::primary_graph::{
    application_contribution::QualifiedOutputSource, WorthQueryOutputDemandDenial,
    WorthQueryOutputDemandDenialKind as Kind,
};

mod supporting_sources;

use super::{
    invalidation::InvalidationEditAdmission, partition_index::SourcePartitionPublication,
    ProductCoordinate, RecordedSettlementIdentity, SemanticSource,
    WorthQueryApplicationOutputLineage, WorthQueryCurrentOutputCandidate,
};

/// Namespace coverage is installed by the native output-family owner.
/// Recovery invalidates it before admitting any restored publication.
#[derive(Default)]
pub(super) struct SourcePublicationCoverage {
    installed: bool,
    reestablishment_required: bool,
}

impl SourcePublicationCoverage {
    pub(super) fn install(&mut self) {
        assert!(!self.installed);
        self.installed = true;
    }

    pub(super) fn invalidate(&mut self) {
        self.reestablishment_required = true;
    }

    fn complete(&self) -> bool {
        self.installed && !self.reestablishment_required
    }
}

/// Borrowed native classification; retaining it requires separate admission.
pub(in crate::domain_computation::primary_graph) struct CoveredSourcePublication<'source> {
    source: &'source QualifiedOutputSource,
    publication: SourcePublication,
    candidate: Option<WorthQueryCurrentOutputCandidate>,
    aliases: Vec<(SemanticSource, [u8; 32])>,
    primary_alias_count: usize,
}

pub(in crate::domain_computation::primary_graph) enum SourcePublication {
    AwaitingFirst,
    /// Heads are not filtered by output entity or fact-bearing posture.
    /// The ordinary family/currentness owner resolves their actual meaning.
    Published(Vec<std::sync::Arc<RecordedSettlementIdentity>>),
}

impl CoveredSourcePublication<'_> {
    pub(in crate::domain_computation::primary_graph) fn source(&self) -> &QualifiedOutputSource {
        self.source
    }

    pub(in crate::domain_computation::primary_graph) fn publication(&self) -> &SourcePublication {
        &self.publication
    }

    pub(in crate::domain_computation::primary_graph) fn primary_aliases(
        &self,
    ) -> &[(SemanticSource, [u8; 32])] {
        &self.aliases[..self.primary_alias_count]
    }

    pub(in crate::domain_computation::primary_graph) fn take_candidate(
        &mut self,
    ) -> Option<WorthQueryCurrentOutputCandidate> {
        self.candidate.take()
    }

    pub(in crate::domain_computation::primary_graph) fn aliases(
        &self,
    ) -> &[(SemanticSource, [u8; 32])] {
        &self.aliases
    }
}

impl WorthQueryApplicationOutputLineage {
    /// Native checkpoint owners call this before restoring a row. No stored
    /// omission can reset a completed source to AwaitingFirst.
    pub(in crate::domain_computation::primary_graph) fn invalidate_source_publication_coverage(
        &mut self,
    ) {
        self.source_publication_coverage.invalidate();
    }

    /// Visit the exact source partition of each installed family binding and
    /// its branch ancestry. Cost is bounded by that inventory and retained
    /// ancestry; it never enumerates unrelated source partitions.
    pub(in crate::domain_computation::primary_graph) fn covered_source_publication<'source>(
        &self,
        source: &'source QualifiedOutputSource,
        admission: &mut InvalidationEditAdmission,
    ) -> Result<CoveredSourcePublication<'source>, WorthQueryOutputDemandDenial> {
        charge(admission, 1)?;
        let address = source.address();
        let bindings = self
            .output_families
            .get(address.family())
            .ok_or_else(|| stop(Kind::ProducerUnavailable))?;
        let bytes = bindings
            .len()
            .checked_mul(
                std::mem::size_of::<std::sync::Arc<RecordedSettlementIdentity>>()
                    + std::mem::size_of::<SemanticSource>(),
            )
            .and_then(|bytes| u64::try_from(bytes).ok())
            .ok_or_else(|| stop(Kind::RetentionBudgetExceeded))?;
        admission
            .admit_read_scratch(bytes)
            .map_err(|_| stop(Kind::RetentionBudgetExceeded))?;
        let mut published = Vec::with_capacity(bindings.len());
        let mut publications = super::family_selection::PublicationHeads::default();
        let mut aliases = Vec::with_capacity(bindings.len());
        for (binding, role) in bindings {
            charge(admission, 1)?;
            let native_source = SemanticSource {
                runtime_authority: address.runtime(),
                schema: address.schema().clone(),
                scope: address.scope(),
                output_binding: self
                    .binding_identity(*binding)
                    .ok_or_else(|| stop(Kind::ProducerUnavailable))?,
            };
            aliases.push((native_source.clone(), address.partition()));
            let mut coordinate = ProductCoordinate {
                occurrence: address.occurrence(),
                generation: source.generation(),
            };
            let mut depth = 0usize;
            loop {
                charge(admission, 1)?;
                match self.source_partition_publication(
                    &native_source,
                    coordinate,
                    address.partition(),
                    admission,
                )? {
                    SourcePartitionPublication::Vacant => {}
                    SourcePartitionPublication::Prepared => {
                        return Err(stop(Kind::SchedulingDeferred))
                    }
                    SourcePartitionPublication::Published(head) => {
                        published.push(std::sync::Arc::clone(&head.recorded.settlement_identity));
                        charge(admission, publications.admission_work() as u64)?;
                        publications.insert(Some(address.partition()), role, *binding, depth, head);
                        break;
                    }
                }
                let Some(parent) = self.origins.get(&coordinate.occurrence).copied() else {
                    break;
                };
                coordinate = parent;
                depth = depth
                    .checked_add(1)
                    .ok_or_else(|| stop(Kind::WorkBudgetExceeded))?;
            }
        }
        if published.is_empty() && (source.lifecycle() != crate::domain_computation::primary_graph::application_contribution::WorthQueryProducerLifecyclePosture::Initial
            || !self.source_publication_coverage.complete()) {
            return Err(stop(Kind::IncompleteDependencyCoverage));
        }
        let (heads, ambiguous) = publications.finish();
        if ambiguous || heads.len() > 1 {
            return Err(stop(Kind::IncompleteDependencyCoverage));
        }
        let candidate = match heads.as_slice() {
            [] => None,
            [(role, head)] => Some(
                WorthQueryCurrentOutputCandidate::from_recorded(head, role)
                    .ok_or_else(|| stop(Kind::IncompleteDependencyCoverage))?,
            ),
            _ => unreachable!("ambiguous heads were refused"),
        };
        let primary_alias_count = aliases.len();
        if let Some(candidate) = &candidate {
            self.supporting_publication_sources(candidate, &mut aliases, admission)?;
        }
        Ok(CoveredSourcePublication {
            candidate,
            primary_alias_count,
            source,
            aliases,
            publication: if published.is_empty() {
                SourcePublication::AwaitingFirst
            } else {
                SourcePublication::Published(published)
            },
        })
    }
}

fn charge(
    admission: &mut InvalidationEditAdmission,
    work: u64,
) -> Result<(), WorthQueryOutputDemandDenial> {
    admission
        .charge_external_work(work)
        .map_err(|_| stop(Kind::WorkBudgetExceeded))
}

fn stop(kind: Kind) -> WorthQueryOutputDemandDenial {
    WorthQueryOutputDemandDenial::new(kind, "output-source-publication")
}
