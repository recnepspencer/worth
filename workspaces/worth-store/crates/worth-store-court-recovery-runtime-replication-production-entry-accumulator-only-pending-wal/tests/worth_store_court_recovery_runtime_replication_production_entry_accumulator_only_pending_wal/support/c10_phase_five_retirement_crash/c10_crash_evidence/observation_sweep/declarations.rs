//! A killed rewrite recovered under fewer observation bytes than it needs is
//! told that limit and the value it was admitted under, at every limit below
//! the need. Completing a rewrite reads whole pages, spans and extent
//! generations, and its reader running out is the limit too: the media is
//! sound, and no block says otherwise. Every read is charged to the limit:
//! the recovery that fits reports reading what the last refusal reached.

pub(crate) use {std::path::Path};

pub(crate) use {worth_store_recovery_runtime::PhysicalRecoveryBlock, worth_store_recovery_runtime::PhysicalRecoveryBlockKind, worth_store_recovery_runtime::PhysicalRecoveryLimitDimension, worth_store_recovery_runtime::PhysicalRecoveryOutcome, worth_store_recovery_runtime::PhysicalRecoveryPlanningDenial, worth_store_recovery_runtime::WorthStoreRecovery};

pub(crate) use {crate::c10_phase_five_retirement_crash::c10_crash_evidence::ordinary_limits_observing};
pub(crate) use {worth_store_testkit_recovery_runtime::phase_three_support::recovery_request_with_limits};

/// How far the sweep moves past a block that names no count its read reached.
pub(crate) const STRIDE: u64 = 512;
pub(crate) const SUFFICIENT_BYTES: u64 = 32 * 1024 * 1024;

/// Sweeps observation bytes from one byte up to the first limit the world
/// recovers under, and returns how many of the blocks on the way the redo
/// plan's completion reported. A block that names the count its read reached
/// is followed by a recovery admitted exactly that count, so every such read
/// is refused once. A blocked recovery has no effect, so the media the kill
/// left answers every probe. Panics, listing them, if any block did not name
/// the limit, or if the recovery that fits read bytes it did not count.
pub(crate) fn completion_blocks_under_every_observation_limit(stage: &str, root: &Path) -> usize {
    let (mut completion_blocks, mut failures, mut admitted) = (0, Vec::new(), 1);
    // The count the last refused read reached, when the sweep stepped to it.
    let mut refused_at = 0;
    while admitted <= SUFFICIENT_BYTES {
        let request = recovery_request_with_limits(root, ordinary_limits_observing(admitted));
        let blocked = match worth_store_recovery_runtime::WorthStoreRecovery::recover(request) {
            worth_store_recovery_runtime::PhysicalRecoveryOutcome::Recovered(handoff) => {
                let observed = handoff.discovery_counters().bytes_observed
                    + handoff.planning_counters().page_extent_bytes();
                drop(handoff);
                assert!(
                    failures.is_empty(),
                    "{stage}: {} blocks did not name the limit:\n{}",
                    failures.len(),
                    failures.join("\n"),
                );
                assert!(
                    refused_at <= observed && observed <= admitted,
                    "{stage}: admitted {admitted} bytes after a read refused at \
                     {refused_at}, recovery counted {observed}",
                );
                return completion_blocks;
            }
            worth_store_recovery_runtime::PhysicalRecoveryOutcome::Blocked(blocked) => blocked,
            outcome => panic!("{stage} {admitted}: neither recovered nor blocked: {outcome:?}"),
        };
        let limit = blocked.cause().limit();
        let named = limit.is_some_and(|limit| {
            limit.dimension() == worth_store_recovery_runtime::PhysicalRecoveryLimitDimension::ObservationBytes
                && limit.admitted() == admitted
                && limit.observed() > admitted
        });
        if !named || blocked.recovery_effects() != 0 {
            failures.push(format!(
                "{admitted}: kind={:?} denial={:?} limit={limit:?} effects={}",
                blocked.cause(),
                blocked.evidence().planning_denial,
                blocked.recovery_effects(),
            ));
        }
        completion_blocks += usize::from(completion_reported(&blocked));
        let reached = limit.map_or(0, |limit| limit.observed());
        (admitted, refused_at) = if reached > admitted + 1 {
            (reached, reached)
        } else {
            (admitted + STRIDE, 0)
        };
    }
    panic!("{stage}: the world did not recover under {SUFFICIENT_BYTES} observation bytes");
}

/// Planning past the selected source's page inventory is completion. Page
/// admission reads that inventory, and completion's successor candidate,
/// which names its denial.
pub(crate) fn completion_reported(blocked: &PhysicalRecoveryBlock) -> bool {
    let evidence = blocked.evidence();
    let inventory = blocked.cause().phase() == worth_store_recovery_runtime::PhysicalRecoveryBlockKind::PageAdmission
        && !matches!(
            evidence.planning_denial,
            Some(worth_store_recovery_runtime::PhysicalRecoveryPlanningDenial::SuccessorCandidate(_))
        );
    evidence.planning_counters.is_some() && !inventory
}