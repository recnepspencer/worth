//! Observes checkpoint-source custody independently of the newer selected
//! post-WAL root. Only the joined controls claim leaves this boundary.

use worth_store_physical_integrity::{ReleaseCustodyHeadWalkBound, ReleaseCustodyHeadWalkDenial};
use worth_store_recovery_physics::{
    PhysicsBound, SelectedCustodyDenial, SelectedHeadRosterAdmissionDenial,
    VerifiedCheckpointReleaseHeadRosterV2, VerifiedSelectedReleaseHeadCustodyV2,
};

use super::super::super::super::{context::PlanningContext, resolved_basis::ResolvedPlanningBasis};
use super::resident_basis;
use crate::entry::{
    PhysicalRecoveryBlockKind, PhysicalRecoveryPlanningDenial,
    PhysicalRecoveryReleaseHeadReadDenial as ReadDenial,
    PhysicalRecoveryReleaseHeadWalkDenial as WalkDenial,
    PhysicalRecoverySelectedReleaseHeadDenial as Denial,
};
use crate::integrity_ingress::{admit_addressed_root, RecoveryArtifactNamespaceJoin};
use crate::orchestration::planning::completion::historical_publication::{
    discovery_failure, remaining_observation, HistoricalFailure,
};
use crate::orchestration::planning::selected_source_inventory::{
    self, ResidentAllowance, ResidentTraceDenial, RoutesFailure,
};
use crate::progression::PlanningCustody;

const ARTIFACT: &str = "checkpoint-source-release-head-v2";

pub(super) fn admit(
    mut context: PlanningContext,
    basis: &mut ResolvedPlanningBasis,
) -> Result<PlanningContext, crate::entry::PhysicalRecoveryOutcome> {
    let remaining_bytes = match remaining_observation(&context, basis) {
        Ok(remaining_bytes) => remaining_bytes,
        Err(failure) => {
            let limit = failure.limit(&context.limits, 0);
            return Err(block(context, basis, Denial::ObservationByteLimit, limit));
        }
    };
    let mut resident = match resident_basis::seed(&context, basis) {
        Ok(resident) => resident,
        Err(limit) => {
            let denial = limit.map_or(Denial::DuplicateClaim, |limit| {
                Denial::ResidentBoundExceeded {
                    required: limit.observed,
                    admitted: limit.admitted,
                }
            });
            return Err(block(context, basis, denial, limit));
        }
    };
    let Some(shared) = context.coordination.owner().checkpoint() else {
        return Err(block(context, basis, Denial::MissingCheckpoint, None));
    };
    let mut discovery = context
        .authority
        .media
        .bounded_discovery(
            crate::orchestration::reader_limit::UNCOUNTED_READS,
            remaining_bytes,
        )
        .expect("positive V2 checkpoint-source bounds");
    let mut scratch = 0;
    let claim = observe(
        &mut discovery,
        &context.selection,
        shared.stream(),
        context.authority.record_format,
        &mut basis.observed_pages.manifest_budget,
        &mut context.integrity_trace,
        &mut scratch,
        &mut resident,
    );
    let counters = discovery.counters();
    context.authority.media = discovery.finish();
    basis.observed_pages.historical_publication_reads = basis
        .observed_pages
        .historical_publication_reads
        .saturating_add(counters.addressed_artifacts_read);
    basis.observed_pages.historical_publication_bytes_read = basis
        .observed_pages
        .historical_publication_bytes_read
        .saturating_add(counters.bytes_read);
    basis
        .observed_pages
        .historical_publication_peak_scratch_bytes = basis
        .observed_pages
        .historical_publication_peak_scratch_bytes
        .max(resident.peak());
    let claim = match claim {
        Ok(claim) => claim,
        Err(denial) => {
            let limit = resident_basis::limit_failure(&context, &resident)
                .or_else(|| limit_of(&denial, &context.limits, remaining_bytes));
            return Err(block(context, basis, denial, limit));
        }
    };
    if !matches!(&basis.custody, PlanningCustody::Unresolved) {
        return Err(block(context, basis, Denial::DuplicateClaim, None));
    }
    basis.custody = PlanningCustody::SourceHeads(claim);
    Ok(context)
}

#[allow(clippy::too_many_arguments)]
fn observe(
    discovery: &mut worth_store::physical_runtime::BoundedRecoveryFilesystemDiscovery,
    selection: &worth_store_recovery_physics::PhysicalSourceSelection,
    stream: &worth_store_physical_integrity::VerifiedCheckpointStream,
    format: worth_store_physical_format::PhysicalRecordFormatDeclaration,
    budget: &mut crate::orchestration::planning::manifest_entry_budget::ManifestEntryBudget,
    trace: &mut crate::integrity_ingress::RecoveryIntegrityIngressTrace,
    scratch: &mut u64,
    resident: &mut ResidentAllowance,
) -> Result<VerifiedSelectedReleaseHeadCustodyV2, Denial> {
    let checkpoint = selection.checkpoint().ok_or(Denial::MissingCheckpoint)?;
    if stream.facts() != *checkpoint.checkpoint() {
        return Err(Denial::Roster(SelectedCustodyDenial::CertificateRoster));
    }
    let generation = checkpoint.checkpoint().source().root().generation();
    resident
        .transient(
            u64::from(format.page_size().bytes())
                .checked_mul(3)
                .ok_or(Denial::WalkLimits)?,
        )
        .map_err(|_| resident_denial(resident))?;
    let root_unit = budget
        .charge_root()
        .map_err(|_| Denial::ManifestEntryLimit)?;
    let observed = discovery
        .read_root_manifest(generation, u64::from(format.page_size().bytes()))
        .map_err(|failure| Denial::SourceRootRead {
            generation,
            failure,
        })?;
    let admitted = admit_addressed_root(
        RecoveryArtifactNamespaceJoin::from_canonical(&observed),
        discovery.store_identity(),
        format,
        generation,
    )
    .map_err(|denial| Denial::SourceRootIntegrity { generation, denial })?;
    let (root, observed_format) = admitted.project();
    drop(observed);
    if observed_format != format {
        return Err(Denial::SourceRootFormatMismatch);
    }
    let routes = selected_source_inventory::observe_routes_held(
        discovery, &root, format, &root_unit, budget, trace, resident,
    )
    .map_err(|failure| routes_denial(failure, resident))?;
    // Walking the head roster is one lookup, however many blocks hold it.
    budget.consume(1).map_err(|_| Denial::ManifestEntryLimit)?;
    // The lookup is charged; the walk is admitted as one view, of no more
    // entries than recovery admits, whatever earlier phases left of them.
    let entries = budget.admitted();
    // Even an empty remainder must produce the same typed resident denial,
    // before constructing walker metadata or encoding the source root.
    resident
        .transient(
            u64::from(format.page_size().bytes())
                .checked_mul(2)
                .ok_or(Denial::WalkLimits)?,
        )
        .map_err(|_| resident_denial(resident))?;
    let memory = resident.remaining();
    // Entry cardinality and resident storage are separate denial dimensions.
    // The physics owner preflights the declared head backing against memory.
    let maximum_heads = entries;
    // Physics bounds the walk's shape by the verified roster, not by this
    // budget. Each head block is read under its own ceiling, one page; the
    // reader's allowance is the only byte budget, and it says when it ran out.
    let roster = VerifiedCheckpointReleaseHeadRosterV2::admit_checkpoint_source(
        selection,
        stream,
        &root,
        format,
        maximum_heads,
        memory,
        |reference, maximum| {
            resident
                .transient(u64::from(format.page_size().bytes()))
                .map_err(|_| {
                    let (required, admitted) = resident_bounds(resident);
                    ReadDenial::ResidentBoundExceeded {
                        reference,
                        required,
                        admitted,
                    }
                })?;
            let observed = discovery
                .read_release_custody_head_block(reference.generation(), reference.block(), maximum)
                .map_err(|failure| ReadDenial::Media { reference, failure })?;
            let bytes = observed
                .bytes()
                .ok_or(ReadDenial::MissingBytes { reference })?;
            let mut copied = Vec::new();
            copied
                .try_reserve_exact(bytes.len())
                .map_err(|cause| ReadDenial::Allocation {
                    reference,
                    requested: bytes.len() as u64,
                    cause,
                })?;
            copied.extend_from_slice(bytes);
            Ok(copied)
        },
    )
    .map_err(|denial| roster_refused(denial, resident))?;
    resident
        .transient(roster.admission_peak_resident_bytes())
        .map_err(|_| resident_denial(resident))?;
    resident
        .bytes(roster.owned_heap_bytes().ok_or(Denial::WalkLimits)?)
        .map_err(|_| resident_denial(resident))?;
    let controls = super::head_v2_controls::read_checkpoint_source_controls(
        discovery, &routes, &roster, format, budget, trace, scratch, resident,
    )
    .map_err(Denial::Control)?;
    let decode_peak = controls
        .iter()
        .map(|control| control.manifest().bytes().len())
        .max()
        .unwrap_or(0);
    resident
        .transient(
            u64::try_from(decode_peak)
                .map_err(|_| Denial::WalkLimits)?
                .checked_mul(3)
                .and_then(|v| v.checked_add(u64::from(format.page_size().bytes())))
                .ok_or(Denial::WalkLimits)?,
        )
        .map_err(|_| resident_denial(resident))?;
    roster
        .join_controls(&routes, controls)
        .map_err(Denial::ControlJoin)
}

fn resident_bounds(resident: &ResidentAllowance) -> (u64, u64) {
    (
        resident.exceeded_requirement().unwrap_or(u64::MAX),
        resident.used().saturating_add(resident.remaining()),
    )
}

fn resident_denial(resident: &ResidentAllowance) -> Denial {
    let (required, admitted) = resident_bounds(resident);
    Denial::ResidentBoundExceeded { required, admitted }
}

/// Why the source routes went unobserved: what observation says of them, or
/// the room this phase had no allowance to hold. The second is never an
/// entry limit.
fn routes_denial(
    failure: RoutesFailure<ResidentTraceDenial>,
    resident: &ResidentAllowance,
) -> Denial {
    match failure {
        RoutesFailure::Observation(failure) => Denial::SourceRoutes(failure.evidence()),
        RoutesFailure::Held(ResidentTraceDenial::ResidentBoundExceeded) => {
            resident_denial(resident)
        }
        // The head walk's allocation denial is the one this phase can name.
        RoutesFailure::Held(ResidentTraceDenial::Allocation { requested, cause }) => {
            Denial::HeadWalk(WalkDenial::Allocation { requested, cause })
        }
    }
}

/// What physics refused of the roster or of the walk under it.
fn roster_refused(
    denial: SelectedHeadRosterAdmissionDenial<ReadDenial>,
    resident: &mut ResidentAllowance,
) -> Denial {
    match denial {
        SelectedHeadRosterAdmissionDenial::Custody(SelectedCustodyDenial::Limit(past))
            if past.dimension() == PhysicsBound::ResidentBytes =>
        {
            // Preserve the owner's requested local window alongside the
            // already-live aggregate seed; do not relabel authority failures.
            let _ = resident.transient(past.observed());
            Denial::ResidentBoundExceeded {
                required: past.observed(),
                admitted: past.admitted(),
            }
        }
        SelectedHeadRosterAdmissionDenial::Custody(denial) => Denial::Roster(denial),
        // The verified roster counts more heads than recovery admits entries.
        SelectedHeadRosterAdmissionDenial::HeadEntries { observed, admitted } => {
            Denial::RosterEntryLimit { observed, admitted }
        }
        SelectedHeadRosterAdmissionDenial::Allocation { requested, cause } => {
            Denial::HeadWalk(WalkDenial::Allocation { requested, cause })
        }
        SelectedHeadRosterAdmissionDenial::Walk(denial) => Denial::HeadWalk(match denial {
            ReleaseCustodyHeadWalkDenial::Read(denial) => WalkDenial::Read(denial),
            // The tree holds more heads than the verified roster counts.
            ReleaseCustodyHeadWalkDenial::Visit(()) => WalkDenial::EntryCountExceeded,
            ReleaseCustodyHeadWalkDenial::Storage(denial) => WalkDenial::Read(denial),
            ReleaseCustodyHeadWalkDenial::Format(denial) => WalkDenial::Format(denial),
            ReleaseCustodyHeadWalkDenial::Root => WalkDenial::Root,
            ReleaseCustodyHeadWalkDenial::DuplicateNode => WalkDenial::DuplicateNode,
            ReleaseCustodyHeadWalkDenial::BoundExceeded => WalkDenial::BoundExceeded,
            // The node bound is the verified roster's shape, not a budget of
            // ours.
            ReleaseCustodyHeadWalkDenial::Limit(past) => match past.dimension() {
                ReleaseCustodyHeadWalkBound::Nodes => WalkDenial::RosterBlockCeiling {
                    observed: past.observed(),
                    admitted: past.admitted(),
                },
                ReleaseCustodyHeadWalkBound::ResidentBytes => WalkDenial::ResidentBoundExceeded {
                    required: past.observed(),
                    admitted: past.admitted(),
                },
            },
            ReleaseCustodyHeadWalkDenial::Allocation { requested, cause } => {
                WalkDenial::Allocation { requested, cause }
            }
        }),
    }
}

fn block(
    context: PlanningContext,
    basis: &ResolvedPlanningBasis,
    denial: Denial,
    limit: Option<crate::entry::PhysicalRecoveryLimitFailure>,
) -> crate::entry::PhysicalRecoveryOutcome {
    context.block_with_planning_attempt_denial(
        PhysicalRecoveryBlockKind::SelectedCustody,
        basis.planning_counters(),
        ARTIFACT,
        limit,
        PhysicalRecoveryPlanningDenial::SelectedReleaseHead(denial),
    )
}

/// The limit a denied observation names, with the value that passed it. A
/// roster of more heads than recovery admits entries carries its own count.
fn limit_of(
    denial: &Denial,
    limits: &crate::entry::PhysicalRecoveryLimitDeclaration,
    remaining_bytes: u64,
) -> Option<crate::entry::PhysicalRecoveryLimitFailure> {
    match *denial {
        Denial::RosterEntryLimit { observed, admitted } => {
            Some(crate::entry::PhysicalRecoveryLimitFailure {
                dimension: crate::entry::PhysicalRecoveryLimitDimension::ManifestEntries,
                observed,
                admitted,
            })
        }
        _ => unread(denial)?.limit(limits, remaining_bytes),
    }
}

/// What a denied observation says about recovery's limits.
fn unread(denial: &Denial) -> Option<HistoricalFailure> {
    use crate::entry::PhysicalRecoveryPageAdmissionDenial as Page;
    use crate::entry::PhysicalRecoveryReleaseHeadControlDenial as Control;
    match denial {
        Denial::ManifestEntryLimit
        | Denial::SourceRoutes(Page::ManifestEntryLimit)
        | Denial::HeadWalk(WalkDenial::Read(ReadDenial::ManifestEntryLimit { .. }))
        | Denial::Control(Control::ManifestEntryLimit) => Some(HistoricalFailure::ManifestEntries),
        Denial::ObservationByteLimit | Denial::SourceRoutes(Page::ObservationByteLimit) => {
            Some(HistoricalFailure::ObservationBytes(None))
        }
        Denial::SourceRootRead { failure, .. }
        | Denial::HeadWalk(WalkDenial::Read(ReadDenial::Media { failure, .. })) => {
            Some(discovery_failure(failure.clone()))
        }
        Denial::Control(Control::ControlRead { denial, .. }) => {
            Some(HistoricalFailure::from(denial.clone()))
        }
        _ => None,
    }
}

#[cfg(test)]
#[path = "head_v2_tests.rs"]
mod tests;
