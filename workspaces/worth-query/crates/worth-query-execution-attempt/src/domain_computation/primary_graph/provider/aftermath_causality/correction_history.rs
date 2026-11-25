//! Exact lineage postings and consumption reads; never whole-history reconstruction.
use super::*;
use crate::domain_computation::application_aftermath::WorthQueryRecordedCorrectionSource;
use worth_foundational::facade::AspectFieldLocator;
use worth_query_declaration::archive_decode_authority::ARCHIVE_DECODE;
use worth_query_execution_lineage::facade::ports::PrimaryGraphIntegrationHandleInternalPort as _;
use worth_relational::facade::indexes::{BoundedEntityFieldBoundary, DerivedIndexId};
pub mod page_allocation;
pub use page_allocation::WorthQueryCorrectionHistoryPageAllocation;

#[derive(Debug)]
pub struct WorthQueryCorrectionHistoryBoundary {
    native: BoundedEntityFieldBoundary,
}
#[derive(Debug)]
struct WorthQueryCorrectionHistoryPage {
    pub entries: Vec<WorthQueryCanonicalCorrectionFact>,
    pub next: Option<WorthQueryCorrectionHistoryBoundary>,
    pub examined_entry_count: usize,
    pub allocation: WorthQueryCorrectionHistoryPageAllocation,
}
pub struct WorthQueryCorrectionHistoryProgress {
    pub entries: Vec<WorthQueryCanonicalCorrectionFact>,
    pub progress: WorthQueryActiveReversalProgress,
    pub examined_entry_count: usize,
    pub consumption_examined_entry_count: usize,
    pub allocation: WorthQueryCorrectionHistoryPageAllocation,
}
struct ExactCorrectionFactLookup {
    fact: Option<WorthQueryCanonicalCorrectionFact>,
    examined_entry_count: usize,
}
#[derive(Debug)]
pub enum WorthQueryActiveReversalProgress {
    Complete(Option<WorthQueryCanonicalCorrectionFact>),
    Incomplete {
        boundary: WorthQueryCorrectionHistoryBoundary,
        active: Option<WorthQueryCanonicalCorrectionFact>,
    },
}

impl WorthQueryPrimaryGraphProvider {
    pub fn reversal_of_source_at_basis(
        &self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        basis: &worth_relational::facade::branch::AdmittedRelationalBranchBasis,
        source: &WorthQueryRecordedCorrectionSource,
    ) -> Result<Option<WorthQueryCanonicalCorrectionFact>, WorthQueryAftermathCausalityReadDenial>
    {
        let layout = self.graph.layout_ref().provider_aftermath_causality();
        self.lookup_correction_fact(
            execution,
            basis,
            layout.lineage.reversed_source_index,
            &layout.lineage.reversed_source,
            lineage_fields::selector(&source.coordinates()),
            source.world_owner_coordinate(),
        )
        .map(|lookup| lookup.fact)
    }
    pub fn consumption_of_reversal_at_basis(
        &self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        basis: &worth_relational::facade::branch::AdmittedRelationalBranchBasis,
        reversal: &WorthQueryRecordedCorrectionSource,
    ) -> Result<Option<WorthQueryCanonicalCorrectionFact>, WorthQueryAftermathCausalityReadDenial>
    {
        let fact = self
            .consumption_of_native_reversal_at_basis(
                execution,
                basis,
                reversal.relational(),
                reversal.world_owner_coordinate(),
            )?
            .fact;
        if fact
            .as_ref()
            .is_some_and(|fact| fact.active.as_ref() != Some(&reversal.coordinates()))
        {
            return Err(WorthQueryAftermathCausalityReadDenial::InconsistentCanonicalRelationships);
        }
        Ok(fact)
    }
    fn consumption_of_native_reversal_at_basis(
        &self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        basis: &worth_relational::facade::branch::AdmittedRelationalBranchBasis,
        reversal: &worth_relational::facade::history::RelationalCommitReceipt,
        owner: worth_foundational::facade::WorldOwnerCoordinate,
    ) -> Result<ExactCorrectionFactLookup, WorthQueryAftermathCausalityReadDenial> {
        let layout = self.graph.layout_ref().provider_aftermath_causality();
        self.lookup_correction_fact(
            execution,
            basis,
            layout.lineage.consumed_reversal_index,
            &layout.lineage.consumed_reversal,
            lineage_fields::consumption_selector(reversal),
            owner,
        )
    }
    fn lookup_correction_fact(
        &self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        basis: &worth_relational::facade::branch::AdmittedRelationalBranchBasis,
        index: DerivedIndexId,
        locator: &AspectFieldLocator,
        value: AspectValue,
        owner: worth_foundational::facade::WorldOwnerCoordinate,
    ) -> Result<ExactCorrectionFactLookup, WorthQueryAftermathCausalityReadDenial> {
        self.with_correction_snapshot(execution, basis, index, |runtime, snapshot, layout| {
            let request = BoundedEntityFieldLookupRequest::new(
                snapshot.clone(),
                index,
                layout.entity_kind,
                locator.clone(),
                value,
                2,
            )
            .map_err(|_| WorthQueryAftermathCausalityReadDenial::Unavailable)?;
            let rows = runtime
                .index_access()
                .execute_bounded_entity_field_lookup(
                    request,
                    BoundedIndexParityMode::Production,
                    execution,
                )
                .map_err(lookup_denial)?;
            if rows.overflowed() || rows.candidate_entity_ids().len() > 1 {
                return Err(
                    WorthQueryAftermathCausalityReadDenial::InconsistentCanonicalRelationships,
                );
            }
            let fact = rows
                .candidate_entity_ids()
                .first()
                .map(|entity| {
                    canonical_record::read(runtime, snapshot, layout, execution, *entity, owner)
                })
                .transpose()?;
            Ok(ExactCorrectionFactLookup {
                fact,
                examined_entry_count: rows.examined_entry_count(),
            })
        })
    }
    /// O(page width) posting work plus exact reads for the returned relevant facts.
    fn correction_alternatives_at_basis(
        &self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        basis: &worth_relational::facade::branch::AdmittedRelationalBranchBasis,
        root: &WorthQueryRecordedCorrectionSource,
        maximum: usize,
        after: Option<&WorthQueryCorrectionHistoryBoundary>,
    ) -> Result<WorthQueryCorrectionHistoryPage, WorthQueryAftermathCausalityReadDenial> {
        let index = self
            .graph
            .layout_ref()
            .provider_aftermath_causality()
            .lineage
            .reversal_root_index;
        self.with_correction_snapshot(execution, basis, index, |runtime, snapshot, layout| {
            let view = runtime
                .read_truth()
                .project_snapshot(snapshot)
                .ok_or(WorthQueryAftermathCausalityReadDenial::Unavailable)?;
            let prepared = runtime
                .index_access()
                .prepare_entity_field_lookup(
                    &view,
                    layout.lineage.reversal_root_index,
                    layout.entity_kind,
                    &layout.lineage.reversed_root,
                    execution,
                )
                .map_err(lookup_denial)?;
            let page = prepared
                .execute_prepared_entity_field_page(
                    &lineage_fields::selector(&root.coordinates()),
                    maximum,
                    after.map(|after| &after.native),
                    execution,
                    || Ok::<_, ()>(()),
                )
                .map_err(page_denial)?;
            let mut allocation = WorthQueryCorrectionHistoryPageAllocation::new(execution)?;
            let mut entries = allocation
                .reserve_projection_slots::<WorthQueryCanonicalCorrectionFact>(
                    page.candidate_entity_ids().len(),
                )?;
            for entity in page.candidate_entity_ids() {
                let decoded = canonical_record::read(
                    runtime,
                    snapshot,
                    layout,
                    execution,
                    *entity,
                    root.world_owner_coordinate(),
                )?;
                entries.push(allocation.clone_fact(&decoded)?);
            }
            let examined_entry_count = page.examined_entry_count();
            Ok(WorthQueryCorrectionHistoryPage {
                entries,
                allocation,
                examined_entry_count,
                next: page
                    .into_next_boundary()
                    .map(|native| WorthQueryCorrectionHistoryBoundary { native }),
            })
        })
    }
    pub fn correction_alternatives_progress_at_basis(
        &self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        basis: &worth_relational::facade::branch::AdmittedRelationalBranchBasis,
        root: &WorthQueryRecordedCorrectionSource,
        maximum: usize,
        prior: Option<&WorthQueryActiveReversalProgress>,
    ) -> Result<WorthQueryCorrectionHistoryProgress, WorthQueryAftermathCausalityReadDenial> {
        let (after, active) = match prior {
            None => (None, None),
            Some(WorthQueryActiveReversalProgress::Incomplete { boundary, active }) => {
                (Some(boundary), active.as_ref())
            }
            Some(WorthQueryActiveReversalProgress::Complete(_)) => {
                return Err(WorthQueryAftermathCausalityReadDenial::ForeignContinuation)
            }
        };
        let mut page =
            self.correction_alternatives_at_basis(execution, basis, root, maximum, after)?;
        let mut active = active
            .map(|fact| page.allocation.clone_fact(fact))
            .transpose()?;
        let mut consumption_examined_entry_count = 0usize;
        for entry in &page.entries {
            let lookup = self.consumption_of_native_reversal_at_basis(
                execution,
                basis,
                &entry.child,
                root.world_owner_coordinate(),
            )?;
            consumption_examined_entry_count = consumption_examined_entry_count.checked_add(lookup.examined_entry_count)
                .ok_or_else(|| aftermath_read_stop(worth_relational::facade::transactions::RelationalExecutionDenialCause::ChargedBytesOverflow))?;
            let consumption = lookup.fact;
            if consumption.as_ref().is_some_and(|fact| {
                fact.root != entry.root
                    || fact.active.as_ref().map(|active| active.relational()) != Some(&entry.child)
            }) {
                return Err(
                    WorthQueryAftermathCausalityReadDenial::InconsistentCanonicalRelationships,
                );
            }
            if consumption.is_none() {
                if active.is_some() {
                    return Err(
                        WorthQueryAftermathCausalityReadDenial::InconsistentCanonicalRelationships,
                    );
                }
                active = Some(page.allocation.clone_fact(entry)?);
            }
        }
        Ok(WorthQueryCorrectionHistoryProgress {
            entries: page.entries,
            examined_entry_count: page.examined_entry_count,
            consumption_examined_entry_count,
            allocation: page.allocation,
            progress: match page.next {
                Some(boundary) => WorthQueryActiveReversalProgress::Incomplete { boundary, active },
                None => WorthQueryActiveReversalProgress::Complete(active),
            },
        })
    }
    pub fn performed_correction_source(
        &self,
        fact: &WorthQueryCanonicalCorrectionFact,
    ) -> Option<WorthQueryRecordedCorrectionSource> {
        let completed = self.observe_completed_application(&fact.child)?;
        Some(WorthQueryRecordedCorrectionSource::from_untrusted_parts(ARCHIVE_DECODE,
            completed
                .committed_product_publication()
                .composite_commit()
                .clone(),
            fact.child.clone(),
            crate::domain_computation::application_aftermath::WorthQueryCorrectionSourceMember::of_writer(
                completed.commit_evidence().idempotency().authored_stage_identity(),
            ),
        ))
    }
    fn with_correction_snapshot<T>(
        &self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        basis: &worth_relational::facade::branch::AdmittedRelationalBranchBasis,
        index: DerivedIndexId,
        read: impl FnOnce(
            &worth_relational::facade::runtime::RelationalRuntime,
            &worth_relational::facade::snapshots::SnapshotHandle,
            &WorthQueryAftermathCausalityLayout,
        ) -> Result<T, WorthQueryAftermathCausalityReadDenial>,
    ) -> Result<T, WorthQueryAftermathCausalityReadDenial> {
        self.graph.with_runtime(execution, |runtime| {
            execution.consult().map_err(|cause| aftermath_read_stop(cause.into()))?;
            // Admit an authentic carried/known-empty selection through Native's
            // ordinary field preparation; discarded populations still refuse.
            if !crate::domain_computation::primary_graph::index_currency::selected_index_is_current(
                execution,
                runtime,
                basis,
                index,
            )
            .map_err(aftermath_read_stop)?
            {
                runtime
                    .index_authority()
                    .ensure_field_index_for_basis(
                        index,
                        basis,
                        crate::domain_computation::primary_graph::index_maintenance_budget::ordinary_index_maintenance_budget(),
                        execution,
                    )
                    .map_err(|denial| match denial.kind {
                        worth_relational::facade::indexes::DerivedIndexMaintenanceDenialKind::Execution(
                            cause,
                        ) => aftermath_read_stop(cause),
                        _ => WorthQueryAftermathCausalityReadDenial::IndexUnavailable {
                            index_id: index,
                        },
                    })?;
            }
            let snapshot = crate::domain_computation::primary_graph::exact_basis_access::open_exact_basis_snapshot(runtime, basis).map_err(|denial| aftermath_snapshot_denial(denial.into()))?;
            let result = read(runtime, &snapshot, self.graph.layout_ref().provider_aftermath_causality());
            crate::relational_snapshot_release::release_query_snapshot(runtime, &snapshot);
            result
        })
    }
}

fn lookup_denial(
    denial: worth_relational::facade::indexes::BoundedEntityFieldLookupDenial,
) -> WorthQueryAftermathCausalityReadDenial {
    match denial.kind() {
        worth_relational::facade::indexes::BoundedEntityFieldLookupDenialKind::Execution(cause) => {
            aftermath_read_stop(cause)
        }
        worth_relational::facade::indexes::BoundedEntityFieldLookupDenialKind::IndexNotInstalled
        | worth_relational::facade::indexes::BoundedEntityFieldLookupDenialKind::ExactGenerationUnavailable => WorthQueryAftermathCausalityReadDenial::IndexUnavailable { index_id: denial.index_id() },
        _ => WorthQueryAftermathCausalityReadDenial::Unavailable,
    }
}
fn page_denial(
    denial: worth_relational::facade::indexes::BoundedEntityFieldPageStop<()>,
) -> WorthQueryAftermathCausalityReadDenial {
    use worth_relational::facade::indexes::BoundedEntityFieldPageStop;
    match denial {
        BoundedEntityFieldPageStop::Lookup(denial) => lookup_denial(denial),
        BoundedEntityFieldPageStop::ForeignBoundary => {
            WorthQueryAftermathCausalityReadDenial::ForeignContinuation
        }
        BoundedEntityFieldPageStop::Work(cause) => aftermath_read_stop(cause.into()),
        BoundedEntityFieldPageStop::Memory(cause) => aftermath_read_stop(worth_execution::WorkCeilingDenial::Admission(worth_execution::LeaseDenial::MemoryExhausted(cause)).into()),
        BoundedEntityFieldPageStop::AccountingOverflow => aftermath_read_stop(worth_relational::facade::transactions::RelationalExecutionDenialCause::ChargedBytesOverflow),
        BoundedEntityFieldPageStop::Allocation(cause) => match cause.classify() {
            worth_execution::ExecutionAllocationDenialClass::Request(cause) => aftermath_read_stop(cause.into()),
            worth_execution::ExecutionAllocationDenialClass::Physical(_) => aftermath_read_stop(worth_relational::facade::transactions::RelationalExecutionDenialCause::AllocationFailed),
        },
        _ => WorthQueryAftermathCausalityReadDenial::Unavailable,
    }
}
