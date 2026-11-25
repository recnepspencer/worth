//! Bounded newest-first listing of retained authored writes on one exact target.
//!
//! The durable correction-material catalog supplies candidates; each candidate
//! is admitted through ordinary Home source selection (World ancestry of this
//! target plus fresh disclosure for this principal), so the listing never
//! widens what `select_coordinates` would admit. Reversal state comes from the
//! canonical lineage index at this target's basis, not from retained material.
use super::occurrence::CorrectionLineageObservation;
use super::*;
use crate::domain_computation::application_aftermath::{
    WorthQueryAftermathCausalRole, WorthQueryCorrectionSourceCoordinates,
    WorthQueryRecordedCorrectionSource,
};
use crate::domain_computation::primary_graph::provider::{
    WorthQueryActiveReversalProgress, WorthQueryAftermathCausalityReadDenial,
    WorthQueryCanonicalCorrectionFact,
};
use worth_query_execution_query::facade::ports::ApplicationRuntimeCoreInternalPort as _;
use worth_query_execution_runtime::facade::ports::AdvancementPhaseInternalPort as _;
use worth_query_execution_runtime::facade::ports::ProductBranchLeaseInternalPort as _;
use worth_query_execution_runtime::facade::ports::ProductRuntimeInternalPort as _;

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
    HistorySource(RuntimeWorldServiceDenial<worth_runtime_world::facade::WorldReadmissionDenial>),
    /// Source selection refused for a reason that is not about one entry.
    Selection(WorthQueryCorrectionSelectionDenial),
    Lineage(WorthQueryAftermathCausalityReadDenial),
    /// One entry's reversal lineage exceeded the bounded classification walk.
    LineageTooLong,
}

struct Candidate {
    sequence: u64,
    coordinates: WorthQueryCorrectionSourceCoordinates,
}

struct Reapplication {
    root: WorthQueryCorrectionSourceCoordinates,
    occurrence: WorthQueryCorrectionSourceCoordinates,
    sequence: u64,
}

impl<'runtime, S: ApplicationSchema> WorthQuerySelectedProductOperation<'runtime, S> {
    /// Lists at most `limit` correctable authored writes on this target, newest first.
    ///
    /// O(retained material · log) to order candidates, then one Home source
    /// selection per examined candidate and one bounded lineage walk per
    /// listed entry. Candidates this principal cannot select on this target
    /// (other branches, undisclosed, expired or not declared correctable) are
    /// omitted rather than reported.
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
        let candidates = self.newest_first_candidates(phase)?;
        let mut reapplications = Vec::new();
        let mut records = Vec::new();
        for candidate in &candidates {
            if records.len() == limit.get() {
                break;
            }
            let selected = match self.select_home_correction_source(
                phase,
                principal,
                request,
                &candidate.coordinates,
                maximum_history,
                maximum_results,
            ) {
                Ok(selected) => selected,
                Err(cause) if omits_entry(&cause) => continue,
                Err(cause) => return Err(WorthQueryCorrectionHistoryDenial::Selection(cause)),
            };
            if let Some(lineage) = selected.occurrence().causality() {
                note_reapplication(&mut reapplications, &lineage, candidate);
                continue;
            }
            let binding = selected.occurrence().original_binding().ok_or(
                WorthQueryCorrectionHistoryDenial::Selection(
                    WorthQueryCorrectionSelectionDenial::SourceNotCorrectable,
                ),
            )?;
            let root = super::lineage::source_locator(&selected);
            let active = self.active_reversal(phase, &root)?;
            let (state, handle) = classify(candidate, active, &reapplications, &candidates);
            records.push(WorthQueryCorrectionHistoryRecord {
                original: candidate.coordinates.clone(),
                handle,
                binding,
                sequence: candidate.sequence,
                state,
            });
        }
        Ok(records)
    }

    fn newest_first_candidates(
        &self,
        phase: &WorthQueryAdvancementPhase<'_>,
    ) -> Result<Vec<Candidate>, WorthQueryCorrectionHistoryDenial> {
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
        let retained = application
            .inner
            .runtime
            .primary_provider_ref()
            .retained_correction_history_candidates()
            .map_err(D::Material)?;
        let originals = owner
            .inspection_port()
            .observe_program_activation_history(&execution)
            .map_err(D::HistorySource)?;
        let mut candidates = retained
            .into_iter()
            .filter_map(|writer| {
                let sequence = originals
                    .original_state(writer.original)?
                    .reserved_commit()
                    .ordinal();
                Some(Candidate {
                    sequence,
                    coordinates: WorthQueryCorrectionSourceCoordinates::from_untrusted_parts(
                        world_owner,
                        sequence,
                        writer.commit,
                        writer.member,
                    ),
                })
            })
            .collect::<Vec<_>>();
        candidates.sort_by(|left, right| {
            (right.sequence, right.coordinates.member())
                .cmp(&(left.sequence, left.coordinates.member()))
        });
        Ok(candidates)
    }

    /// The root's unconsumed reversal at this target's basis, if any.
    fn active_reversal(
        &self,
        phase: &WorthQueryAdvancementPhase<'_>,
        root: &WorthQueryRecordedCorrectionSource,
    ) -> Result<Option<WorthQueryCanonicalCorrectionFact>, WorthQueryCorrectionHistoryDenial> {
        use WorthQueryCorrectionHistoryDenial as D;
        let application = self.application();
        let execution = phase
            .execution_request_for(application.inner.runtime.product_runtime_ref())
            .map_err(|_| D::ForeignPhase)?;
        let provider = application.inner.runtime.primary_provider_ref();
        let mut prior = None;
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
            match page.progress {
                WorthQueryActiveReversalProgress::Complete(active) => return Ok(active),
                incomplete => prior = Some(incomplete),
            }
        }
        Err(D::LineageTooLong)
    }
}

/// Selection refusals that describe one candidate, not the listing itself.
fn omits_entry(cause: &WorthQueryCorrectionSelectionDenial) -> bool {
    use WorthQueryCorrectionSelectionDenial as D;
    match cause {
        D::Material(_)
        | D::ExactHistory(_)
        | D::HistoryLimit
        | D::SourceUnavailable
        | D::SourceNotCorrectable
        | D::SourcePublicationMismatch
        | D::SourceMemberUnavailable
        | D::Disclosure(_)
        | D::Ancestry(_) => true,
        D::StaleIncarnation | D::HistorySource(_) | D::ForeignPhase | D::ForeignRuntime => false,
    }
}

/// Newest-first scanning meets a lineage's latest reapplication before its root.
fn note_reapplication(
    reapplications: &mut Vec<Reapplication>,
    lineage: &CorrectionLineageObservation<'_>,
    candidate: &Candidate,
) {
    if lineage.role() != WorthQueryAftermathCausalRole::RecordedReapply {
        return;
    }
    let root = lineage.correction_lineage().root().coordinates();
    if reapplications.iter().all(|known| known.root != root) {
        reapplications.push(Reapplication {
            root,
            occurrence: candidate.coordinates.clone(),
            sequence: candidate.sequence,
        });
    }
}

fn classify(
    candidate: &Candidate,
    active: Option<WorthQueryCanonicalCorrectionFact>,
    reapplications: &[Reapplication],
    candidates: &[Candidate],
) -> (
    WorthQueryCorrectionHistoryState,
    WorthQueryCorrectionSourceCoordinates,
) {
    match active {
        Some(reversal) => {
            let at = candidates
                .iter()
                .find(|known| known.coordinates.relational() == &reversal.child)
                .map_or(candidate.sequence, |known| known.sequence);
            (
                WorthQueryCorrectionHistoryState::Reversed { at },
                reversal.source,
            )
        }
        None => match reapplications
            .iter()
            .find(|known| known.root == candidate.coordinates)
        {
            Some(latest) => (
                WorthQueryCorrectionHistoryState::Applied {
                    at: latest.sequence,
                },
                latest.occurrence.clone(),
            ),
            None => (
                WorthQueryCorrectionHistoryState::Applied {
                    at: candidate.sequence,
                },
                candidate.coordinates.clone(),
            ),
        },
    }
}
