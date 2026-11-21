//! One checkpoint capture is one stage of its original request.
//!
//! The control owner documents the rate: one work unit per byte produced by
//! each serializer pass (sizing and emission are each charged), per payload
//! byte hashed and per byte copied; zero-work boundaries are free. Every
//! oracle below is derived from encoded byte counts read independently of the
//! charging code: the checkpoint length, its reported sections, the journal
//! artifacts and a MessagePack walk of the checkpoint's inline members.
//! Reconstructing a region that has no recorded checkpoint is its residency
//! owner's own phase, observed apart from the encoding rate.
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
/// The empty checkpoint's total charge: 2 x 2,645 - 16 + 175, the first court.
const EMPTY_TOTAL: u64 = 5_449;

#[test]
fn checkpoint_stage_charges_the_documented_rate_for_each_encoded_byte() {
    // T = (2 x bytes - magic)          final wire: sizing pass + emission pass
    //   + classified sections          one more sizing pass per section
    //   + sum(digest inputs)           one digest pass per resident envelope
    //   + sum(embedded artifacts)      one locator hash per envelope carried inline
    //   + region capture               a region with no recorded checkpoint
    //
    // CHK23 carries each envelope as its original journal artifact, so no
    // envelope is serialized again: embedding costs its member's locator hash.
    //
    //  0 rows: 2 x 2,645 - 16 + 175 + 0 + 0                   =     5,449
    //  8 rows: 2 x 57,564 - 16 + 46,368 + 44,196 + 43,876     =   249,552 + regions
    // 64 rows: 2 x 425,003 - 16 + 359,570 + 354,880 + 352,320 = 1,916,760 + regions
    // classified = envelopes + roots + cells + mirror + index definitions:
    //   0 + 1 + 172 + 1 + 1; 43,900 + 1,351 + 340 + 776 + 1;
    //   352,512 + 3,646 + 340 + 3,071 + 1.
    for (rows, bytes, sections, digests, artifacts, encoding) in [
        (0, 2_645, [0, 1, 172, 1, 1], 0, 0, EMPTY_TOTAL),
        (
            8,
            57_564,
            [43_900, 1_351, 340, 776, 1],
            44_196,
            43_876,
            249_552,
        ),
        (
            64,
            425_003,
            [352_512, 3_646, 340, 3_071, 1],
            354_880,
            352_320,
            1_916_760,
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
        // have no recorded checkpoint and are each reconstructed once.
        assert_eq!(observed.omitted_phases, 0);
        let regions: Vec<u64> = observed
            .phases
            .iter()
            .filter(|row| row.phase == "region_capture")
            .map(|row| row.charged_work)
            .collect();
        assert_eq!(regions.len(), if rows == 0 { 0 } else { 2 }, "rows={rows}");
        let region: u64 = regions.iter().sum();
        assert_eq!(report.charged_work(), encoding + region, "rows={rows}");
        drop(checkpoint);
        assert_eq!(request.memory().unwrap().charged_memory_bytes(), 0);
    }
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

#[test]
fn checkpoint_reconstruction_scope_growth_is_linear_in_cold_input() {
    // The unchanged fixture grows current records and retained publication
    // metadata together. Cold admission may grow with that input, but must not
    // repeat whole-population scope preparation for every record.
    let capture = |rows| {
        let request = serial_request();
        let execution = ExecutionRequest::serial(&request);
        let runtime = runtime_with_rows(rows);
        let observer = ExecutionScopeSetupObservationForTest::install_on_this_thread_for_test();
        let checkpoint = runtime
            .durability_authority()
            .native_checkpoint(&execution, execution.allocation_policy())
            .unwrap();
        let counts = observer.counts_for_test();
        drop(checkpoint);
        counts
    };
    let (base, doubled) = (capture(32), capture(64));
    for (name, first, second) in [
        ("stage setups", base.stage_setups, doubled.stage_setups),
        (
            "limit derivations",
            base.limit_derivations,
            doubled.limit_derivations,
        ),
        ("admissions", base.admissions, doubled.admissions),
    ] {
        assert!(
            first > 0,
            "cold export must use its original request: {name}"
        );
        assert!(
            (second as u128) * 2 < (first as u128) * 5,
            "{name}: cold scope work outgrows cold input: {first}->{second}"
        );
    }
}
