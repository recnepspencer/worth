//! One checkpoint capture is one stage of its original request.
//!
//! The control owner documents the rate: one work unit per byte produced by
//! each serializer pass (sizing and emission are each charged), per payload
//! byte hashed and per byte copied; zero-work boundaries are free. The
//! encoding oracle is derived from encoded byte counts read independently of
//! the charging code: the checkpoint length, its reported sections, the
//! journal artifacts and a MessagePack walk of the checkpoint's inline members.
//! Capturing a region is its residency owner's own phase, observed apart: the
//! court bounds it by the region images it copies, the records its root check
//! visits and linear growth, not by the encoding rate.
use worth_execution::{
    CancellationSource, ExecutionAllocationDenialClass, ExecutionAllocationPolicy as Allocation,
    ExecutionRequest, ExecutionScopeSetupCountsForTest as Setup,
    ExecutionScopeSetupObservationForTest, ExecutionWorkCeiling, MapKernelStop,
    RequestConsultObservationForTest, SerialRequest, WorkCeilingDenial,
};
use worth_foundational::{
    DeterminismContract, ExecutionBudget, ExecutionPosture, ExecutionRequestPolicy,
};
use worth_relational::facade::{
    durability::{NativeCheckpointSectionBytes, RelationalNativeCheckpointCaptureDenial as Denial},
    identity::{KindId, PartitionId},
    inspection::observe_maintenance_work,
    runtime::RelationalRuntime,
    symbols::ClientKey,
    transactions::{
        CreateIntent, EntitySpec, MutationIntent, RelationalExecutionDenialCause as Cause,
        WorkerIntentBatch,
    },
};
use worth_testkit_relational::aspect_storage::{
    publications::begin_main_transaction, AspectSchemaFixture,
};
use worth_testkit_relational::shared_relational_certification_test_execution_policy::serial_request;

/// The native checkpoint prefix; it is emitted but not sized.
const CHECKPOINT_MAGIC: &[u8] = b"WORTH-REL-CHK24\n";
/// Journal artifact header: magic 8, version 2, position 8, commit 8,
/// digest 32, payload length 8.
const ARTIFACT_HEADER_BYTES: usize = 66;
/// The payload digest's root hashes its domain `worth.relational.canonical.v8\0`,
/// the position, the commit and the three field digests.
const DIGEST_ROOT_BYTES: usize = 30 + 8 + 8 + 3 * 32;
/// A region image member: `WORTH-REL-REG8\n`, then version 2, partition 4,
/// schema digest 32 and kind 1 before its payload.
const REGION_IMAGE_MAGIC: &[u8] = b"WORTH-REL-REG8\n";
const REGION_IMAGE_FRAME_BYTES: usize = 15 + 2 + 4 + 32 + 1;
/// The empty checkpoint's total charge: 2 x 2,506 - 16 + 175, the first court.
const EMPTY_TOTAL: u64 = 5_171;

#[test]
fn checkpoint_stage_charges_the_documented_rate_for_each_encoded_byte() {
    // T = (2 x bytes - magic)          final wire: sizing pass + emission pass
    //   + classified sections          one more sizing pass per section
    //   + sum(digest inputs)           one digest pass per resident envelope
    //   + sum(external artifacts)      one locator hash per canonical member
    //   + region capture               the residency owner's phase, bounded below
    //
    // CHK24 carries each envelope as an external journal artifact, so wire
    // inline members are always empty and `envelopes` section bytes are 0.
    // Authored index writes persist their catalog bound (public #384), so each
    // artifact and digest input is 31 bytes longer than before.
    let mut per_row_reconstruction = Vec::new();
    for (rows, expected_bytes, sections, digests, artifacts) in [
        (0, Some(2_506), [0, 1, 172, 1, 1], 0, 0),
        (8, None, [0, 1_351, 340, 776, 1], 44_444, 44_124),
        (64, None, [0, 3_639, 340, 3_064, 1], 356_864, 354_304),
    ] {
        let request = serial_request();
        let execution = ExecutionRequest::serial(&request);
        let runtime = runtime_with_rows(rows);
        let journal: Vec<Vec<u8>> = runtime
            .durability()
            .canonical_commit_artifacts(&execution)
            .map(|artifact| artifact.unwrap().as_bytes().to_vec())
            .collect();
        let ((checkpoint, report), observed) = observe_maintenance_work(64, || {
            ExecutionWorkCeiling::new(None)
                .run_serial(&request, || {
                    runtime
                        .durability_authority()
                        .native_checkpoint_with_policy(&execution, Allocation::SystemAllocation, worth_relational::facade::durability::RelationalCheckpointArtifactPolicy::ExternalMembers { current_members: &[], source: None })
                        .unwrap()
                })
                .unwrap()
        });
        let [envelope_section, roots, cells, mirror, indexes] = sections;
        let classified: usize = sections.iter().sum();
        assert!(checkpoint.bytes().starts_with(CHECKPOINT_MAGIC));
        let bytes = checkpoint.bytes().len();
        if let Some(expected) = expected_bytes {
            assert_eq!(bytes, expected, "rows={rows}");
        }
        assert_eq!(
            checkpoint.captured_sections(),
            Some(NativeCheckpointSectionBytes {
                total: bytes,
                envelopes: envelope_section,
                branch_roots: roots,
                branch_cells: cells,
                partition_mirror: mirror,
                derived_indexes: indexes,
                framing_and_metadata: bytes - classified,
            }),
            "rows={rows}"
        );
        let digest_inputs: usize = journal
            .iter()
            .map(|artifact| digest_input_bytes(&artifact[ARTIFACT_HEADER_BYTES..]))
            .sum();
        assert_eq!(digest_inputs, digests, "rows={rows}");
        assert!(inline_members(checkpoint.bytes()).is_empty(), "rows={rows}");
        // External canonical members are exactly the journal's artifacts, in order.
        let external_envelopes: Vec<&[u8]> = checkpoint
            .members()
            .iter()
            .filter_map(|member| match member {
                worth_relational::facade::durability::RelationalCheckpointMember::New {
                    bytes,
                    ..
                } if bytes.starts_with(b"WRTHRCCA") => Some(bytes.bytes()),
                _ => None,
            })
            .collect();
        assert_eq!(external_envelopes, journal, "rows={rows}");
        let external_bytes: usize = external_envelopes.iter().map(|member| member.len()).sum();
        assert_eq!(external_bytes, artifacts, "rows={rows}");
        let encoding = 2 * bytes as u64 - CHECKPOINT_MAGIC.len() as u64
            + classified as u64
            + digests as u64
            + artifacts as u64;
        if rows == 0 {
            assert_eq!(encoding, EMPTY_TOTAL);
        }
        // With rows, the main root's region and the partition mirror's region
        // have no recorded checkpoint, and each capture reconstructs the same
        // two images: it copies each image's payload (the member less its
        // 54-byte frame). The root's capture also checks each record against
        // its selected root, one unit a record.
        assert_eq!(observed.omitted_phases, 0);
        let regions: Vec<u64> = observed
            .phases
            .iter()
            .filter(|row| row.phase == "region_capture")
            .map(|row| row.charged_work)
            .collect();
        let copies: u64 = checkpoint
            .members()
            .iter()
            .filter_map(|member| match member {
                worth_relational::facade::durability::RelationalCheckpointMember::New {
                    bytes,
                    ..
                } if bytes.starts_with(REGION_IMAGE_MAGIC) => {
                    Some((bytes.len() - REGION_IMAGE_FRAME_BYTES) as u64)
                }
                _ => None,
            })
            .sum();
        let region = match regions[..] {
            [] => {
                assert_eq!((rows, copies), (0, 0));
                0
            }
            [root, mirror] => {
                assert_eq!(root, mirror + rows as u64, "rows={rows}");
                assert!(mirror > copies, "rows={rows}: {mirror} <= {copies}");
                per_row_reconstruction.push((rows as u64, mirror - copies));
                root + mirror
            }
            _ => panic!("rows={rows}: {regions:?}"),
        };
        assert_eq!(report.charged_work(), encoding + region, "rows={rows}");
        drop(checkpoint);
        assert_eq!(request.memory().unwrap().charged_memory_bytes(), 0);
    }
    // Reconstruction beyond the copies stays linear in rows: from 8 to 64 rows
    // its work per row grows by less than a quarter.
    let [(few, small), (many, large)] = per_row_reconstruction[..] else {
        panic!("two populated captures: {per_row_reconstruction:?}")
    };
    assert!(
        4 * large * few < 5 * small * many,
        "{small}/{few} -> {large}/{many}"
    );
}

#[test]
fn checkpoint_capture_sets_up_exactly_one_stage_whatever_the_row_count() {
    for rows in [0, 8, 64] {
        let request = serial_request();
        let execution = ExecutionRequest::serial(&request);
        let runtime = runtime_with_rows(rows);
        let observer = ExecutionScopeSetupObservationForTest::install_on_this_thread_for_test();
        let checkpoint = runtime
            .durability_authority()
            .native_checkpoint_with_policy(&execution, Allocation::SystemAllocation, worth_relational::facade::durability::RelationalCheckpointArtifactPolicy::ExternalMembers { current_members: &[], source: None })
            .unwrap();
        let expected = Setup {
            stage_setups: 1,
            limit_derivations: 1,
            admissions: 1,
        };
        assert_eq!(observer.counts_for_test(), expected, "rows={rows}");
        drop(checkpoint);
    }
}

#[test]
fn empty_checkpoint_work_ceiling_bounds_the_whole_capture() {
    // Consult ordinals are charging structure, so the accepted run fixes them:
    // its last charge needs the whole total. A refused run charges exactly the
    // accepted run's prefix up to the first charge whose cumulative requirement
    // exceeds its ceiling; that charge latches the stop, the stage's next
    // consult observes the same origin and the capture consults nothing more.
    let mut accepted = Vec::new();
    for ceiling in [EMPTY_TOTAL, EMPTY_TOTAL - 1, 1, 0] {
        let request = SerialRequest::from_policy(
            &ExecutionRequestPolicy::new(
                ExecutionPosture::Serial,
                DeterminismContract::CanonicalBitwise,
                ExecutionBudget::new(std::num::NonZeroUsize::MIN, None, Some(ceiling)),
            ),
            CancellationSource::new().token(),
            None,
        );
        let execution = ExecutionRequest::serial(&request);
        let runtime = runtime_with_rows(0);
        let main = runtime.main_branch_identity();
        let before = runtime.observe_branch(&main).unwrap().0;
        let observer = RequestConsultObservationForTest::install_on_this_thread_for_test(|_| {});
        let result = runtime
            .durability_authority()
            .native_checkpoint_with_policy(&execution, Allocation::SystemAllocation, worth_relational::facade::durability::RelationalCheckpointArtifactPolicy::ExternalMembers { current_members: &[], source: None });
        let consults = observer.count_for_test();
        let origins = observer.stop_origins_for_test();
        let charges = observer.charged_prefixes_for_test();
        drop(observer);
        if ceiling == EMPTY_TOTAL {
            assert_eq!(origins, []);
            assert_eq!(
                charges.last().map(|(_, required)| *required),
                Some(EMPTY_TOTAL)
            );
            assert!(charges
                .last()
                .is_some_and(|(ordinal, _)| *ordinal < consults));
            accepted = charges;
            let checkpoint = result.expect("the whole capture fits its total charge");
            assert_eq!(checkpoint.bytes().len(), 2_506);
            // A work ceiling changes nothing the checkpoint retains.
            assert_eq!(
                request.memory().unwrap().charged_memory_bytes(),
                memory_outcome(false, 1 << 20, false).unwrap().1
            );
            drop(checkpoint);
        } else {
            let refused = accepted
                .iter()
                .position(|(_, required)| *required > ceiling)
                .unwrap();
            let stop = accepted[refused].0;
            assert_eq!(charges, accepted[..=refused], "ceiling={ceiling}");
            assert_eq!(
                origins,
                [(stop, stop), (stop + 1, stop)],
                "ceiling={ceiling}"
            );
            assert_eq!(consults, stop + 1, "ceiling={ceiling}");
            assert!(
                matches!(result, Err(Denial::Request(Cause::WorkExhausted))),
                "ceiling={ceiling}: {result:?}"
            );
        }
        assert_eq!(request.memory().unwrap().charged_memory_bytes(), 0);
        let after = runtime.observe_branch(&main).unwrap().0;
        assert_eq!(after, before, "ceiling={ceiling}");
    }
}

mod fixture;
mod memory;
use fixture::{digest_input_bytes, inline_members, memory_outcome, runtime_with_rows};

#[test]
fn checkpoint_reconstruction_preserves_complete_identity_and_data() {
    for rows in [0, 8, 64] {
        let request = serial_request();
        let execution = ExecutionRequest::serial(&request);
        let runtime = runtime_with_rows(rows);
        let before = runtime
            .durability_authority()
            .checkpoint(&execution)
            .unwrap();
        let checkpoint = runtime
            .durability_authority()
            .native_checkpoint_with_policy(&execution, execution.allocation_policy(), worth_relational::facade::durability::RelationalCheckpointArtifactPolicy::ExternalMembers { current_members: &[], source: None })
            .unwrap();
        let after = runtime
            .durability_authority()
            .checkpoint(&execution)
            .unwrap();
        // Export may not change any source identity, data, envelope or derived
        // commitment. This is the complete portable image, including private fields.
        assert_eq!(before, after, "source changed during export: rows={rows}");
        let mut restored = AspectSchemaFixture::default().build_runtime();
        restored
            .durability_recovery()
            .restore_native_checkpoint(
                &execution,
                &checkpoint,
                worth_testkit_relational::cold_recovery::members_source(&checkpoint),
            )
            .unwrap();
        let reopened = restored
            .durability_authority()
            .checkpoint(&execution)
            .unwrap();
        // Cold import mints fresh runtime affinity, while the actual arena
        // identities, values, MVCC history and selected commit remain exact.
        assert_eq!(
            before.partition_images, reopened.partition_images,
            "rows={rows}"
        );
        assert_eq!(before.coverage, reopened.coverage);
        assert_eq!(before.symbol_table, reopened.symbol_table);
        assert_eq!(before.aspect_contracts, reopened.aspect_contracts);
        assert_eq!(before.lineage, reopened.lineage);
        assert_eq!(before.index_definitions, reopened.index_definitions);
        drop((before, after, reopened, checkpoint, runtime, restored));
        assert_eq!(request.memory().unwrap().charged_memory_bytes(), 0);
    }
}

/// Synchronously cancel `cancel` on the `cancel_on_nth_consult`-th request
/// consult observed while `budget` is at or above `threshold` bytes, returning
/// `(result, observed_consults_at_or_above_threshold)`.
pub(crate) fn cancel_on_consult_at_memory_threshold<R>(
    budget: worth_execution::SerialMemoryBudget,
    threshold: u64,
    cancel_on_nth_consult: usize,
    cancel: CancellationSource,
    body: impl FnOnce() -> R,
) -> (R, usize) {
    assert!(cancel_on_nth_consult >= 1);
    let seen = std::sync::Arc::new(std::sync::atomic::AtomicUsize::new(0));
    let count = std::sync::Arc::clone(&seen);
    let _observer = RequestConsultObservationForTest::install_on_this_thread_for_test(move |_| {
        if budget.charged_memory_bytes() >= threshold {
            let next = count.fetch_add(1, std::sync::atomic::Ordering::AcqRel) + 1;
            if next == cancel_on_nth_consult {
                cancel.cancel();
            }
        }
    });
    let result = body();
    (result, seen.load(std::sync::atomic::Ordering::Acquire))
}
