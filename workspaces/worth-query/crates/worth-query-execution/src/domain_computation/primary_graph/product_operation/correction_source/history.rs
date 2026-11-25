//! Bounded newest-first listing of retained authored writes on one exact target.
//!
//! One walk of the target's exact product history, from its observed head back
//! at most `maximum_history` parent links, visits only commits of this branch
//! (each page consults the request). Commits with ready retained material are
//! admitted through ordinary Home source selection, handed the identity the
//! walk already resolved, so the listing never widens what `select_coordinates`
//! admits. Reversal state comes from the canonical lineage index; reversal
//! order comes from the walked World ordinals, never from optional material.
use super::cold_selection::WalkedOriginal;
use super::occurrence::CorrectionLineageObservation;
use super::*;
use crate::domain_computation::application_aftermath::{
    WorthQueryAftermathCausalRole, WorthQueryCorrectionSourceCoordinates,
    WorthQueryCorrectionSourceMember, WorthQueryRecordedCorrectionSource,
};
use crate::domain_computation::primary_graph::provider::{
    WorthQueryActiveReversalProgress, WorthQueryAftermathCausalityReadDenial,
    WorthQueryCanonicalCorrectionFact,
};
use std::collections::BTreeMap;
use worth_query_execution_query::facade::ports::ApplicationRuntimeCoreInternalPort as _;
use worth_query_execution_runtime::facade::ports::AdvancementPhaseInternalPort as _;
use worth_query_execution_runtime::facade::ports::ProductBranchLeaseInternalPort as _;
use worth_query_execution_runtime::facade::ports::ProductRuntimeInternalPort as _;
use worth_relational::facade::history::RelationalCommitReceipt;
use worth_runtime_world::facade::{CompositeCommitIdentity, ProductExactHistoryDenial};

mod omission;
use omission::omits_entry;

/// Lineage postings read per page while classifying one entry's reversal.
const LINEAGE_PAGE_ENTRIES: usize = 32;
/// Pages read per entry before the lineage is refused as too long to classify.
const MAXIMUM_LINEAGE_PAGES: usize = 64;

/// Whether a listed authored write is currently in effect on the target.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum WorthQueryCorrectionHistoryState {
    /// The write is in effect; `at` orders its latest application (original or redo).
    Applied { at: u64 },
    /// An unconsumed reversal undid the write; `at` orders that reversal.
    Reversed { at: u64 },
}

/// One retained authored write the principal may currently correct on this target.
///
/// Descriptive: `handle` names the occurrence an undo or redo must select now
/// (the original, its latest reapplication, or the source of its active
/// reversal). Selecting it again freshly checks disclosure and ancestry.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct WorthQueryCorrectionHistoryRecord {
    original: WorthQueryCorrectionSourceCoordinates,
    handle: WorthQueryCorrectionSourceCoordinates,
    binding: std::any::TypeId,
    sequence: u64,
    state: WorthQueryCorrectionHistoryState,
}

impl WorthQueryCorrectionHistoryRecord {
    /// The original ordinary occurrence this entry describes.
    pub fn original(&self) -> &WorthQueryCorrectionSourceCoordinates {
        &self.original
    }
    /// The occurrence an undo or redo of this entry selects at this target.
    pub fn handle(&self) -> &WorthQueryCorrectionSourceCoordinates {
        &self.handle
    }
    /// The original's mutation binding, for typed correction dispatch.
    pub fn source_binding_type(&self) -> std::any::TypeId {
        self.binding
    }
    /// The original's durable World commit ordinal; larger is newer.
    pub fn sequence(&self) -> u64 {
        self.sequence
    }
    pub fn state(&self) -> WorthQueryCorrectionHistoryState {
        self.state
    }
}

/// Why a correction history listing was refused as a whole.
#[derive(Debug)]
pub enum WorthQueryCorrectionHistoryDenial {
    /// Only a Directory Home retains correction material across restarts.
    DurableHomeRequired,
    ForeignPhase,
    Material(crate::facade::correction_material::QueryCorrectionMaterialDenial),
    /// Walking this target's exact history stopped (cancellation, deadline,
    /// work or memory, a retired branch, or an unavailable owner).
    ExactHistory(RuntimeWorldServiceDenial<ProductExactHistoryDenial>),
    /// Source selection refused for a reason that is not about one entry.
    Selection(WorthQueryCorrectionSelectionDenial),
    Lineage(WorthQueryAftermathCausalityReadDenial),
    /// One entry's reversal lineage exceeded the bounded classification walk.
    LineageTooLong,
    /// An active reversal is not a commit of this target's walked history.
    ReversalOutsideHistory,
}

/// One walked commit: its World identity, ordinal and parent-link distance.
struct Walked {
    identity: CompositeCommitIdentity,
    commit: RelationalCommitReceipt,
    distance: usize,
}

/// The caller's history and disclosure bounds, applied to each selection.
///
/// Only World ancestry is tightened to the walk distance; lineage reads back
/// to an older root keep the caller's full history bound.
#[derive(Clone, Copy)]
struct ListingBounds {
    history: Option<NonZeroUsize>,
    results: NonZeroUsize,
}

/// Reapplication and reversal facts gathered newest-first during the walk.
#[derive(Default)]
struct WalkFacts {
    /// Relational commit id to walked World ordinal, for canonical ordering.
    ordinals: BTreeMap<u64, u64>,
    /// Lineage root to its newest reapplication occurrence and ordinal.
    reapplications: BTreeMap<
        (u64, WorthQueryCorrectionSourceMember),
        (WorthQueryCorrectionSourceCoordinates, u64),
    >,
}

fn root_key(
    coordinates: &WorthQueryCorrectionSourceCoordinates,
) -> (u64, WorthQueryCorrectionSourceMember) {
    (coordinates.relational().commit_id.0, coordinates.member())
}

impl<'runtime, S: ApplicationSchema> WorthQuerySelectedProductOperation<'runtime, S> {
    /// Lists at most `limit` correctable authored writes on this target, newest first.
    ///
    /// O(walked commits · log retained) plus, per walked writer with ready
    /// material, one Home source selection bounded by its walk distance and,
    /// per listed entry, one bounded lineage walk. Writers this principal
    /// cannot select (undisclosed, expired or not declared correctable) are
    /// omitted; every request, resource or infrastructure stop propagates.
    pub fn correction_history(
        &self,
        _: &crate::publication_boundary::WorthQueryProgramPublicationAccess,
        phase: &WorthQueryAdvancementPhase<'_>,
        principal: &WorthQueryAuthenticatedExternalPrincipal<S>,
        request: &WorthQueryRequestScope,
        limit: NonZeroUsize,
        maximum_history: Option<NonZeroUsize>,
        maximum_results: NonZeroUsize,
    ) -> Result<Vec<WorthQueryCorrectionHistoryRecord>, WorthQueryCorrectionHistoryDenial> {
        use WorthQueryCorrectionHistoryDenial as D;
        let application = self.application();
        let execution = phase
            .execution_request_for(application.inner.runtime.product_runtime_ref())
            .map_err(|_| D::ForeignPhase)?;
        let owner = application.inner.runtime.product_runtime_ref().owner_ref();
        let home = owner
            .owner_identity()
            .durable()
            .ok_or(D::DurableHomeRequired)?;
        let world_owner = worth_foundational::facade::WorldOwnerCoordinate::Home { owner: home };
        let mut ready = BTreeMap::<u64, Vec<_>>::new();
        for writer in application
            .inner
            .runtime
            .primary_provider_ref()
            .retained_correction_history_candidates()
            .map_err(D::Material)?
        {
            ready
                .entry(writer.commit.commit_id.0)
                .or_default()
                .push(writer);
        }
        let observation = owner.observation_port();
        let mut cursor = observation
            .open_exact_product_history(execution, self.product().observation())
            .map_err(D::ExactHistory)?;
        let mut facts = WalkFacts::default();
        let mut records = Vec::new();
        let mut distance = 0_usize;
        while records.len() < limit.get() {
            let page = observation
                .prepare_exact_product_history_page(execution, &cursor)
                .map_err(D::ExactHistory)?;
            let walked = page.commit().and_then(|commit| {
                let observed = commit.basis().relational_basis().observation();
                let receipt = observed.commit_receipt()?.clone();
                Some(Walked {
                    identity: commit.identity().clone(),
                    commit: receipt,
                    distance,
                })
            });
            let complete = page.is_complete()
                || maximum_history
                    .is_some_and(|maximum| distance.saturating_add(1) >= maximum.get());
            if !complete {
                let (_, next) = cursor
                    .accept_page(page)
                    .map_err(|cause| D::ExactHistory(RuntimeWorldServiceDenial::Denied(cause)))?;
                cursor = next;
            } else {
                drop(page);
            }
            if let Some(walked) = walked {
                facts
                    .ordinals
                    .insert(walked.commit.commit_id.0, walked.identity.ordinal());
                let writers = ready.get(&walked.commit.commit_id.0).into_iter().flatten();
                for writer in writers.filter(|writer| writer.commit == walked.commit) {
                    if records.len() == limit.get() {
                        break;
                    }
                    let coordinates = WorthQueryCorrectionSourceCoordinates::new(
                        world_owner,
                        walked.identity.ordinal(),
                        walked.commit.clone(),
                        writer.member,
                    );
                    if let Some(record) = self.list_writer(
                        phase,
                        principal,
                        request,
                        &walked,
                        coordinates,
                        &mut facts,
                        ListingBounds {
                            history: maximum_history,
                            results: maximum_results,
                        },
                    )? {
                        records.push(record);
                    }
                }
            }
            if complete {
                break;
            }
            distance = distance.saturating_add(1);
        }
        Ok(records)
    }

    /// Selects one walked writer and, when it is an original, classifies it.
    fn list_writer(
        &self,
        phase: &WorthQueryAdvancementPhase<'_>,
        principal: &WorthQueryAuthenticatedExternalPrincipal<S>,
        request: &WorthQueryRequestScope,
        walked: &Walked,
        coordinates: WorthQueryCorrectionSourceCoordinates,
        facts: &mut WalkFacts,
        bounds: ListingBounds,
    ) -> Result<Option<WorthQueryCorrectionHistoryRecord>, WorthQueryCorrectionHistoryDenial> {
        let bound = NonZeroUsize::MIN.saturating_add(walked.distance);
        let selected = match self.select_home_correction_source(
            phase,
            principal,
            request,
            &coordinates,
            None,
            bounds.history,
            bounds.results,
            Some(WalkedOriginal {
                identity: walked.identity.clone(),
                parent_links_bound: bound,
            }),
        ) {
            Ok(selected) => selected,
            Err(cause) if omits_entry(&cause) => return Ok(None),
            Err(cause) => return Err(WorthQueryCorrectionHistoryDenial::Selection(cause)),
        };
        if let Some(lineage) = selected.occurrence().causality() {
            note_reapplication(facts, &lineage, &coordinates, walked.identity.ordinal());
            return Ok(None);
        }
        let Some(binding) = selected.occurrence().original_binding() else {
            return Ok(None);
        };
        let root = super::lineage::source_locator(&selected);
        let (active, reversed_before) = self.active_reversal(phase, &root)?;
        let sequence = walked.identity.ordinal();
        let (state, handle) = match active {
            Some(reversal) => {
                let at = *facts
                    .ordinals
                    .get(&reversal.child.commit_id.0)
                    .ok_or(WorthQueryCorrectionHistoryDenial::ReversalOutsideHistory)?;
                (
                    WorthQueryCorrectionHistoryState::Reversed { at },
                    reversal.source,
                )
            }
            None if !reversed_before => (
                WorthQueryCorrectionHistoryState::Applied { at: sequence },
                coordinates.clone(),
            ),
            // Every reversal was reapplied; undo names the newest reapplication.
            // If its retained material expired the entry is no longer correctable.
            None => match facts.reapplications.get(&root_key(&coordinates)) {
                Some((latest, at)) => (
                    WorthQueryCorrectionHistoryState::Applied { at: *at },
                    latest.clone(),
                ),
                None => return Ok(None),
            },
        };
        Ok(Some(WorthQueryCorrectionHistoryRecord {
            original: coordinates,
            handle,
            binding,
            sequence,
            state,
        }))
    }

    /// The root's unconsumed reversal at this basis, and whether it was ever reversed.
    fn active_reversal(
        &self,
        phase: &WorthQueryAdvancementPhase<'_>,
        root: &WorthQueryRecordedCorrectionSource,
    ) -> Result<(Option<WorthQueryCanonicalCorrectionFact>, bool), WorthQueryCorrectionHistoryDenial>
    {
        use WorthQueryCorrectionHistoryDenial as D;
        let application = self.application();
        let execution = phase
            .execution_request_for(application.inner.runtime.product_runtime_ref())
            .map_err(|_| D::ForeignPhase)?;
        let provider = application.inner.runtime.primary_provider_ref();
        let mut prior = None;
        let mut reversed = false;
        for _ in 0..MAXIMUM_LINEAGE_PAGES {
            let page = provider
                .correction_alternatives_progress_at_basis(
                    &execution,
                    self.product().relational_basis(),
                    root,
                    LINEAGE_PAGE_ENTRIES,
                    prior.as_ref(),
                )
                .map_err(D::Lineage)?;
            reversed |= !page.entries.is_empty();
            match page.progress {
                WorthQueryActiveReversalProgress::Complete(active) => {
                    return Ok((active, reversed))
                }
                incomplete => prior = Some(incomplete),
            }
        }
        Err(D::LineageTooLong)
    }
}

/// Newest-first walking meets a lineage's latest reapplication before its root.
fn note_reapplication(
    facts: &mut WalkFacts,
    lineage: &CorrectionLineageObservation<'_>,
    occurrence: &WorthQueryCorrectionSourceCoordinates,
    ordinal: u64,
) {
    if lineage.role() != WorthQueryAftermathCausalRole::RecordedReapply {
        return;
    }
    let root = lineage.correction_lineage().root().coordinates();
    facts
        .reapplications
        .entry(root_key(&root))
        .or_insert_with(|| (occurrence.clone(), ordinal));
}
