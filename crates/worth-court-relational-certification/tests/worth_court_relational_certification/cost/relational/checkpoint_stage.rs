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
    durability::{
        NativeCheckpointSectionBytes, RelationalDurableMemberLocator,
        RelationalNativeCheckpointCaptureDenial as Denial,
    },
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
const CHECKPOINT_MAGIC: &[u8] = b"WORTH-REL-CHK23\n";
/// Journal artifact header: magic 8, version 2, position 8, commit 8,
/// digest 32, payload length 8.
const ARTIFACT_HEADER_BYTES: usize = 66;
/// The payload digest's root hashes its domain `worth.relational.canonical.v8\0`,
/// the position, the commit and the three field digests.
const DIGEST_ROOT_BYTES: usize = 30 + 8 + 8 + 3 * 32;
/// A region image member: `WORTH-REL-REG7\n`, then version 2, partition 4,
/// schema digest 32 and kind 1 before its payload.
const REGION_IMAGE_MAGIC: &[u8] = b"WORTH-REL-REG7\n";
const REGION_IMAGE_FRAME_BYTES: usize = 15 + 2 + 4 + 32 + 1;
/// The empty checkpoint's total charge: 2 x 2,645 - 16 + 175, the first court.
const EMPTY_TOTAL: u64 = 5_449;

#[test]
fn checkpoint_stage_charges_the_documented_rate_for_each_encoded_byte() {
    // T = (2 x bytes - magic)          final wire: sizing pass + emission pass
    //   + classified sections          one more sizing pass per section
    //   + sum(digest inputs)           one digest pass per resident envelope
    //   + sum(embedded artifacts)      one locator hash per envelope carried inline
    //   + region capture               the residency owner's phase, bounded below
    //
    // CHK23 carries each envelope as its original journal artifact, so no
    // envelope is serialized again: embedding costs its member's locator hash.
    //
    //  0 rows: 2 x 2,645 - 16 + 175 + 0 + 0                   =     5,449
    //  8 rows: 2 x 57,826 - 16 + 46,616 + 44,444 + 44,124     =   250,820 + regions
    // 64 rows: 2 x 426,949 - 16 + 361,554 + 356,864 + 354,304 = 1,926,604 + regions
    // classified = envelopes + roots + cells + mirror + index definitions:
    //   0 + 1 + 172 + 1 + 1; 44,148 + 1,351 + 340 + 776 + 1;
    //   354,496 + 3,646 + 340 + 3,071 + 1.
    //
    // Each envelope's authored index writes persist their catalog bound
    // (public #384): `Covered { covered_next_index_id, entries }` replaces
    // `Complete(entries)`, 42 encoded bytes instead of 11 for an empty packet,
    // so every artifact, digest input and embedded member is 31 bytes longer.
    // The rest of the checkpoint's change (+14 bytes at 8 rows, -38 at 64) is
    // the MessagePack width of the changed artifact digests' bytes.
    let mut per_row_reconstruction = Vec::new();
    for (rows, bytes, sections, digests, artifacts, encoding) in [
        (0, 2_645, [0, 1, 172, 1, 1], 0, 0, EMPTY_TOTAL),
        (
            8,
            57_826,
            [44_148, 1_351, 340, 776, 1],
            44_444,
            44_124,
            250_820,
        ),
        (
            64,
            426_949,
            [354_496, 3_646, 340, 3_071, 1],
            356_864,
            354_304,
            1_926_604,
        ),
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
                        .native_checkpoint(&execution, Allocation::SystemAllocation)
                        .unwrap()
                })
                .unwrap()
        });
        let [envelope_section, roots, cells, mirror, indexes] = sections;
        let classified: usize = sections.iter().sum();
        assert!(checkpoint.bytes().starts_with(CHECKPOINT_MAGIC));
        assert_eq!(checkpoint.bytes().len(), bytes, "rows={rows}");
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
        // The embedded envelopes are exactly the journal's artifacts, in order;
        // their section adds a 3-byte bin16 header to each.
        let embedded: Vec<&[u8]> = inline_members(checkpoint.bytes())
            .into_iter()
            .filter(|member| member.starts_with(b"WRTHRCCA"))
            .collect();
        assert_eq!(embedded, journal, "rows={rows}");
        let embedded_bytes: usize = embedded.iter().map(|member| member.len()).sum();
        assert_eq!(embedded_bytes, artifacts, "rows={rows}");
        assert_eq!(envelope_section, artifacts + 3 * journal.len());
        let derived = 2 * bytes as u64 - CHECKPOINT_MAGIC.len() as u64
            + classified as u64
            + digests as u64
            + artifacts as u64;
        assert_eq!(derived, encoding, "rows={rows}: the arithmetic above");
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
        let copies: u64 = inline_members(checkpoint.bytes())
            .into_iter()
            .filter(|member| member.starts_with(REGION_IMAGE_MAGIC))
            .map(|member| (member.len() - REGION_IMAGE_FRAME_BYTES) as u64)
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
            .native_checkpoint(&execution, Allocation::SystemAllocation)
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
            .native_checkpoint(&execution, Allocation::SystemAllocation);
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
            assert_eq!(checkpoint.bytes().len(), 2_645);
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
            .native_checkpoint(&execution, execution.allocation_policy())
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
            .restore_native_checkpoint(&execution, &checkpoint)
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
