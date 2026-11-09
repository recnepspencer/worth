//! One checkpoint capture is one stage of its original request.
//!
//! The control owner documents the rate: one work unit per byte produced by
//! each serializer pass (sizing and emission are each charged), per payload
//! byte hashed and per byte copied; zero-work boundaries are free. Every
//! oracle below is derived from encoded byte counts read independently of the
//! charging code: the checkpoint length, its reported sections, the journal
//! artifact payloads and a MessagePack walk of the embedded envelopes.
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

/// `WORTH-REL-CHK15\n`: the prefix is emitted but not sized.
const CHECKPOINT_MAGIC_BYTES: u64 = 16;
/// Journal artifact header: magic 8, version 2, position 8, commit 8,
/// digest 32, payload length 8.
const ARTIFACT_HEADER_BYTES: usize = 66;
/// The empty checkpoint's total charge, derived in the first court.
const EMPTY_TOTAL: u64 = 2_376;

#[test]
fn checkpoint_stage_charges_the_documented_rate_for_each_encoded_byte() {
    // T = (2 x bytes - magic)          final wire: sizing pass + emission pass
    //   + classified sections          one more sizing pass per section
    //   + sum(journal payloads)        one digest pass per resident envelope
    //   + 2 x sum(embedded envelopes)  sizing + emission per carried envelope
    //
    //  0 rows: 2 x 1,108 - 16 + 176 + 0 + 2 x 0                       =     2,376
    //  8 rows: 2 x 36,807 - 16 + 26,846 + 24,402 + 2 x 25,522         =   175,890
    // 64 rows: 2 x 263,373 - 16 + 207,226 + 196,763 + 2 x 205,723     = 1,342,165
    // classified = envelopes + roots + cells + mirror + index definitions:
    //   1 + 1 + 172 + 1 + 1; 25,547 + 514 + 371 + 413 + 1;
    //   205,918 + 516 + 376 + 415 + 1.
    for (rows, bytes, sections, payloads, envelopes, total) in [
        (0, 1_108, [1, 1, 172, 1, 1], 0, 0, 2_376),
        (
            8,
            36_807,
            [25_547, 514, 371, 413, 1],
            24_402,
            25_522,
            175_890,
        ),
        (
            64,
            263_373,
            [205_918, 516, 376, 415, 1],
            196_763,
            205_723,
            1_342_165,
        ),
    ] {
        let request = serial_request();
        let execution = ExecutionRequest::serial(&request);
        let runtime = runtime_with_rows(rows);
        let journal: usize = runtime
            .durability()
            .canonical_commit_artifacts(&execution)
            .map(|artifact| artifact.unwrap().as_bytes().len() - ARTIFACT_HEADER_BYTES)
            .sum();
        let (checkpoint, report) = ExecutionWorkCeiling::new(None)
            .run_serial(&request, || {
                runtime
                    .durability_authority()
                    .native_checkpoint(&execution, Allocation::SystemAllocation)
                    .unwrap()
            })
            .unwrap();
        let [envelope_section, roots, cells, mirror, indexes] = sections;
        let classified: usize = sections.iter().sum();
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
        assert_eq!(journal, payloads, "rows={rows}");
        let embedded: usize = embedded_envelope_lengths(checkpoint.bytes()).iter().sum();
        assert_eq!(embedded, envelopes, "rows={rows}");
        let derived = 2 * bytes as u64 - CHECKPOINT_MAGIC_BYTES
            + classified as u64
            + payloads as u64
            + 2 * envelopes as u64;
        assert_eq!(derived, total, "rows={rows}: the arithmetic above");
        assert_eq!(report.charged_work(), total, "rows={rows}");
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
    // 574 of the 585 consults charge work. The last three charges are the
    // one-byte sizing writes of the mirror, envelope and index-definition
    // sections; the last one needs the whole total.
    let mut accepted = Vec::new();
    for (ceiling, consults, origins) in [
        (EMPTY_TOTAL, 585, &[][..]),
        (EMPTY_TOTAL - 1, 582, &[(581, 581), (582, 581)][..]),
        (1, 4, &[(3, 3), (4, 3)][..]),
        (0, 3, &[(2, 2), (3, 2)][..]),
    ] {
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
        assert_eq!(observer.count_for_test(), consults, "ceiling={ceiling}");
        assert_eq!(
            observer.stop_origins_for_test(),
            origins,
            "ceiling={ceiling}"
        );
        let charges = observer.charged_prefixes_for_test();
        drop(observer);
        if ceiling == EMPTY_TOTAL {
            assert_eq!(charges.len(), 574);
            assert_eq!(
                charges[charges.len() - 3..],
                [(577, 2_374), (579, 2_375), (581, EMPTY_TOTAL)]
            );
            accepted = charges;
            let checkpoint = result.expect("the whole capture fits its total charge");
            assert_eq!(checkpoint.bytes().len(), 1_108);
            assert_eq!(request.memory().unwrap().charged_memory_bytes(), 1_108);
            drop(checkpoint);
        } else {
            // The refused write is the first whose cumulative charge exceeds
            // the ceiling; every earlier write charged as in the accepted run.
            let refused = accepted
                .iter()
                .position(|(_, required)| *required > ceiling)
                .unwrap();
            assert_eq!(charges, accepted[..=refused], "ceiling={ceiling}");
            assert_eq!(accepted[refused].0, origins[0].0, "ceiling={ceiling}");
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

#[test]
fn empty_checkpoint_memory_boundary_is_its_bytes_plus_one_stage() {
    let _guard = crate::leased_preparation::ordinary_context::serial();
    // 1,508 = the 1,108 checkpoint bytes + 400 bytes the one stage retains,
    // on serial and leased lineages alike.
    for leased in [false, true] {
        let fits = memory_outcome(leased, 1_508, false);
        assert!(matches!(fits, Ok(1_108)), "leased={leased}: {fits:?}");
        let short = memory_outcome(leased, 1_507, false);
        assert!(
            matches!(
                short,
                Err(Denial::Request(Cause::PolicyMemoryExhausted {
                    requested: 1_108,
                    admitted: 1_107,
                    ancestor: 0,
                }))
            ),
            "leased={leased}: {short:?}"
        );
        // Cancellation outranks the memory refusal at the first gate.
        for memory in [1_507, 1_508] {
            match (leased, memory_outcome(leased, memory, true)) {
                (false, Err(Denial::Request(Cause::Cancelled))) => {}
                (true, Err(Denial::Allocation(cause))) => assert_eq!(
                    cause.classify(),
                    ExecutionAllocationDenialClass::Request(WorkCeilingDenial::Stopped(
                        MapKernelStop::Cancelled
                    ))
                ),
                (_, outcome) => panic!("leased={leased} memory={memory}: {outcome:?}"),
            }
        }
    }
}

mod fixture;
use fixture::{embedded_envelope_lengths, memory_outcome, runtime_with_rows};

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

#[test]
fn checkpoint_refusals_preserve_request_custody_and_publication() {
    let _guard = crate::leased_preparation::ordinary_context::serial();
    for leased in [false, true] {
        assert!(memory_outcome(leased, 1 << 20, false).unwrap() > 0);
        assert!(matches!(
            memory_outcome(leased, 0, false),
            Err(Denial::Request(Cause::PolicyMemoryExhausted { .. })) | Err(Denial::Allocation(_))
        ));
        match (leased, memory_outcome(leased, 0, true)) {
            (false, Err(Denial::Request(Cause::Cancelled))) => {}
            (true, Err(Denial::Allocation(cause))) => assert_eq!(
                cause.classify(),
                ExecutionAllocationDenialClass::Request(WorkCeilingDenial::Stopped(
                    MapKernelStop::Cancelled
                ))
            ),
            (_, outcome) => panic!("cancellation must outrank memory refusal: {outcome:?}"),
        }
    }
    let request = SerialRequest::from_policy(
        &ExecutionRequestPolicy::new(
            ExecutionPosture::Serial,
            DeterminismContract::CanonicalBitwise,
            ExecutionBudget::new(std::num::NonZeroUsize::MIN, None, Some(0)),
        ),
        CancellationSource::new().token(),
        None,
    );
    let execution = ExecutionRequest::serial(&request);
    let runtime = runtime_with_rows(0);
    let before = runtime
        .observe_branch(&runtime.main_branch_identity())
        .unwrap()
        .0;
    assert!(matches!(
        runtime
            .durability_authority()
            .native_checkpoint(&execution, Allocation::SystemAllocation),
        Err(Denial::Request(Cause::WorkExhausted))
    ));
    assert_eq!(request.memory().unwrap().charged_memory_bytes(), 0);
    assert_eq!(
        runtime
            .observe_branch(&runtime.main_branch_identity())
            .unwrap()
            .0,
        before
    );
}
