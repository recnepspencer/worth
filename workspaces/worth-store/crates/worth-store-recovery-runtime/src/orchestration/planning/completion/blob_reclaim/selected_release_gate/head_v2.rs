//! Observes checkpoint-source custody independently of the newer selected
//! post-WAL root. Only the joined controls claim leaves this boundary.

use worth_store_physical_integrity::{
    ReleaseCustodyHeadWalkDenial, ReleaseCustodyHeadWalkLimitsV1,
};
use worth_store_recovery_physics::{
    SelectedCustodyDenial, SelectedHeadRosterAdmissionDenial,
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
use crate::orchestration::planning::selected_source_inventory::{self, ResidentAllowance};
use crate::progression::PlanningCustody;

const ARTIFACT: &str = "checkpoint-source-release-head-v2";

pub(super) fn admit(
    mut context: PlanningContext,
    basis: &mut ResolvedPlanningBasis,
) -> Result<PlanningContext, crate::entry::PhysicalRecoveryOutcome> {
    let remaining_entries = basis.observed_pages.manifest_budget.remaining();
    let remaining_bytes = context
        .limits
        .observation_bytes
        .saturating_sub(context.counters.bytes_observed)
        .saturating_sub(basis.observed_pages.bytes_read)
        .saturating_sub(basis.observed_pages.candidate_bytes_read)
        .saturating_sub(basis.observed_pages.source_copy_bytes_read)
        .saturating_sub(basis.observed_pages.historical_publication_bytes_read);
    if remaining_entries == 0 || remaining_bytes == 0 {
        let denial = if remaining_entries == 0 {
            Denial::ManifestEntryLimit
        } else {
            Denial::ObservationByteLimit
        };
        return Err(block(context, basis, denial, None));
    }
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
        .bounded_discovery(remaining_entries, remaining_bytes)
        .expect("positive V2 checkpoint-source bounds");
    let mut scratch = 0;
    let claim = observe(
        &mut discovery,
        &context.selection,
        shared.stream(),
        context.authority.record_format,
        remaining_bytes,
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
            let limit = resident_basis::limit_failure(&context, &resident);
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
    byte_limit: u64,
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
                .ok_or(Denial::ManifestEntryLimit)?,
        )
        .map_err(|_| resident_denial(resident))?;
    budget.consume(1).map_err(|_| Denial::ManifestEntryLimit)?;
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
    let routes = selected_source_inventory::observe_routes_with_resident_budget(
        discovery, &root, format, budget, trace, resident,
    )
    .map_err(|failure| Denial::SourceRoutes(failure.evidence()))?;
    let entries = budget.remaining();
    // Even an empty remainder must produce the same typed resident denial,
    // before constructing walker metadata or encoding the source root.
    resident
        .transient(
            u64::from(format.page_size().bytes())
                .checked_mul(2)
                .ok_or(Denial::ManifestEntryLimit)?,
        )
        .map_err(|_| resident_denial(resident))?;
    let memory = resident.remaining();
    // Entry cardinality and resident storage are separate denial dimensions.
    // The physics owner preflights the declared head backing against memory.
    let maximum_heads = entries;
    let limits =
        ReleaseCustodyHeadWalkLimitsV1::new(entries.max(1), maximum_heads, byte_limit, memory, 16)
            .ok_or(Denial::WalkLimits)?;
    let roster = match VerifiedCheckpointReleaseHeadRosterV2::admit_checkpoint_source(
        selection,
        stream,
        &root,
        format,
        limits,
        maximum_heads,
        memory,
        |reference, maximum| {
            budget
                .consume(1)
                .map_err(|_| ReadDenial::ManifestEntryLimit { reference })?;
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
    ) {
        Ok(roster) => roster,
        Err(SelectedHeadRosterAdmissionDenial::Custody(
            SelectedCustodyDenial::ResidentBoundExceeded { required, admitted },
        )) => {
            // Preserve the owner's requested local window alongside the
            // already-live aggregate seed; do not relabel authority failures.
            let _ = resident.transient(required);
            return Err(Denial::ResidentBoundExceeded { required, admitted });
        }
        Err(SelectedHeadRosterAdmissionDenial::Custody(denial)) => {
            return Err(Denial::Roster(denial))
        }
        Err(SelectedHeadRosterAdmissionDenial::Allocation { requested, cause }) => {
            return Err(Denial::HeadWalk(WalkDenial::Allocation {
                requested,
                cause,
            }));
        }
        Err(SelectedHeadRosterAdmissionDenial::Walk(denial)) => {
            return Err(Denial::HeadWalk(match denial {
                ReleaseCustodyHeadWalkDenial::Read(denial) => WalkDenial::Read(denial),
                ReleaseCustodyHeadWalkDenial::Visit(()) => WalkDenial::EntryCountExceeded,
                ReleaseCustodyHeadWalkDenial::Storage(denial) => WalkDenial::Read(denial),
                ReleaseCustodyHeadWalkDenial::Format(denial) => WalkDenial::Format(denial),
                ReleaseCustodyHeadWalkDenial::Root => WalkDenial::Root,
                ReleaseCustodyHeadWalkDenial::DuplicateNode => WalkDenial::DuplicateNode,
                ReleaseCustodyHeadWalkDenial::BoundExceeded => WalkDenial::BoundExceeded,
                ReleaseCustodyHeadWalkDenial::Allocation { requested, cause } => {
                    WalkDenial::Allocation { requested, cause }
                }
                ReleaseCustodyHeadWalkDenial::ResidentBoundExceeded { required, admitted } => {
                    WalkDenial::ResidentBoundExceeded { required, admitted }
                }
            }));
        }
    };
    resident
        .transient(roster.admission_peak_resident_bytes())
        .map_err(|_| resident_denial(resident))?;
    resident
        .bytes(
            roster
                .owned_heap_bytes()
                .ok_or(Denial::ManifestEntryLimit)?,
        )
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
                .map_err(|_| Denial::ManifestEntryLimit)?
                .checked_mul(3)
                .and_then(|v| v.checked_add(u64::from(format.page_size().bytes())))
                .ok_or(Denial::ManifestEntryLimit)?,
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
