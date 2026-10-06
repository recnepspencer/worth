use worth_store_physical_format::{
    DurableInlineRecordPlacement, PersistedRecordIdentity, PhysicalGeneration,
    PhysicalGenerationAuthority, PhysicalPageId, PhysicalRecordFormatDeclaration,
    PhysicalRecordSlot, PhysicalRewriteRedo, PhysicalSegmentId, RecordSegmentPageManifestEntry,
};

use super::{verify_inline_coordinates, HistoricalFailure};

struct Coordinates {
    format: PhysicalRecordFormatDeclaration,
    source: DurableInlineRecordPlacement,
    source_entries: Vec<RecordSegmentPageManifestEntry>,
    destination: DurableInlineRecordPlacement,
    result_entries: Vec<RecordSegmentPageManifestEntry>,
}

impl Coordinates {
    fn rewrite(&self, source_generation: u64, source_offset: u64) -> PhysicalRewriteRedo {
        PhysicalRewriteRedo::new(
            [1; 32],
            [2; 32],
            7,
            source_generation,
            source_offset,
            4 * self.format.page_size().bytes(),
            [3; 32],
            4,
            0,
            0,
            [4; 32],
            8,
            9,
            8,
        )
        .unwrap()
    }

    fn verify(&self, rewrite: PhysicalRewriteRedo) -> Result<(), HistoricalFailure> {
        verify_inline_coordinates(
            self.format,
            rewrite,
            self.source,
            &self.source_entries,
            self.destination,
            &self.result_entries,
        )
    }
}

fn coordinates() -> Coordinates {
    let format = PhysicalRecordFormatDeclaration::builder().admit().unwrap();
    let authority = PhysicalGenerationAuthority::for_canonical_physical_format();
    let segment = PhysicalSegmentId::from_raw(1).unwrap();
    let tail = PhysicalPageId::from_raw(4).unwrap();
    let slot = authority
        .slot_cell(segment, tail, PhysicalRecordSlot::from_raw(1).unwrap())
        .with_slot_generation(PhysicalGeneration::from_raw(1).unwrap());
    let record = PersistedRecordIdentity::new([5; 16], 1).unwrap();
    let source_segment = authority
        .segment_cell(segment)
        .with_segment_generation(PhysicalGeneration::from_raw(3).unwrap());
    let result_segment = authority
        .segment_cell(segment)
        .with_segment_generation(PhysicalGeneration::from_raw(4).unwrap());
    let source_tail = authority
        .page_cell(segment, tail)
        .with_page_generation(PhysicalGeneration::from_raw(8).unwrap());
    let result_tail = authority
        .page_cell(segment, tail)
        .with_page_generation(PhysicalGeneration::from_raw(9).unwrap());
    let source = DurableInlineRecordPlacement::legacy_unknown(
        record,
        source_segment,
        source_tail,
        slot,
        16,
        7_500,
    )
    .unwrap();
    let destination = DurableInlineRecordPlacement::legacy_unknown(
        record,
        result_segment,
        result_tail,
        slot,
        16,
        7_500,
    )
    .unwrap();
    let mut source_entries = Vec::new();
    let mut result_entries = Vec::new();
    for index in 0..4 {
        let page = PhysicalPageId::from_raw(index + 1).unwrap();
        let prior_generation = if index == 3 { 8 } else { index + 3 };
        let prior = authority
            .page_cell(segment, page)
            .with_page_generation(PhysicalGeneration::from_raw(prior_generation).unwrap());
        let result = authority
            .page_cell(segment, page)
            .with_page_generation(PhysicalGeneration::from_raw(prior_generation + 1).unwrap());
        source_entries.push(
            RecordSegmentPageManifestEntry::new(prior, source_segment, 6, index as u32 + 2)
                .unwrap(),
        );
        result_entries.push(
            RecordSegmentPageManifestEntry::new(result, result_segment, 4, index as u32).unwrap(),
        );
    }
    Coordinates {
        format,
        source,
        source_entries,
        destination,
        result_entries,
    }
}

#[test]
fn exact_historical_source_and_result_coordinates_admit() {
    let case = coordinates();
    assert!(case
        .verify(case.rewrite(3, 2 * u64::from(case.format.page_size().bytes())))
        .is_ok());
}

#[test]
fn wrong_historical_source_generation_cannot_be_published() {
    let case = coordinates();
    assert!(matches!(
        case.verify(case.rewrite(2, 2 * u64::from(case.format.page_size().bytes()))),
        Err(HistoricalFailure::Invalid)
    ));
}

#[test]
fn source_route_generation_must_match_the_wal_source() {
    let mut case = coordinates();
    let authority = PhysicalGenerationAuthority::for_canonical_physical_format();
    let other_segment = authority
        .segment_cell(PhysicalSegmentId::from_raw(1).unwrap())
        .with_segment_generation(PhysicalGeneration::from_raw(2).unwrap());
    case.source = DurableInlineRecordPlacement::legacy_unknown(
        case.source.record(),
        other_segment,
        case.source.page_cell(),
        case.source.slot_cell(),
        case.source.segment_page_capacity(),
        case.source.payload_bytes(),
    )
    .unwrap();
    assert!(matches!(
        case.verify(case.rewrite(3, 2 * u64::from(case.format.page_size().bytes()))),
        Err(HistoricalFailure::Invalid)
    ));
}

#[test]
fn wrong_historical_source_offset_cannot_be_published() {
    let case = coordinates();
    assert!(matches!(
        case.verify(case.rewrite(3, 3 * u64::from(case.format.page_size().bytes()))),
        Err(HistoricalFailure::Invalid)
    ));
}

#[test]
fn non_tail_page_generation_must_advance_by_one() {
    let mut case = coordinates();
    let authority = PhysicalGenerationAuthority::for_canonical_physical_format();
    let first = authority
        .page_cell(
            PhysicalSegmentId::from_raw(1).unwrap(),
            PhysicalPageId::from_raw(1).unwrap(),
        )
        .with_page_generation(PhysicalGeneration::from_raw(5).unwrap());
    case.result_entries[0] =
        RecordSegmentPageManifestEntry::new(first, case.destination.segment_cell(), 4, 0).unwrap();
    assert!(matches!(
        case.verify(case.rewrite(3, 2 * u64::from(case.format.page_size().bytes()))),
        Err(HistoricalFailure::Invalid)
    ));
}

#[test]
fn collecting_one_placements_pages_is_one_lookup_charged_before_its_reads() {
    use crate::entry::PhysicalRecoveryLimitDimension::ManifestEntries;
    use crate::orchestration::planning::manifest_entry_budget::ManifestEntryBudget;
    use crate::orchestration::planning::page_observation::PageLimit;
    use crate::orchestration::planning::selected_world_fixture::selected_world;
    use crate::orchestration::recovery_budget::recovery_limit_for_test;
    const ADMITTED: u64 = 4_096;
    let placement = coordinates().source;
    let collect = |name: &str, observed: u64| {
        selected_world(name, 4).read(|source| {
            let mut budget = ManifestEntryBudget::new(ADMITTED, observed);
            let mut trace = Default::default();
            let outcome = super::collect(
                source.discovery,
                source.root,
                &mut budget,
                &mut trace,
                &mut 0,
                source.format,
                placement,
                u64::MAX,
                0,
                1,
                None,
            )
            .map(|_| ());
            let reads = source.discovery.counters().addressed_artifacts_read;
            (outcome, budget.remaining(), reads)
        })
    };
    // None left: refused as that limit, before any read.
    assert_eq!(
        collect("inline-lookup-none-left", ADMITTED),
        (
            Err(HistoricalFailure::Limit(PageLimit::Recovery(
                recovery_limit_for_test(ManifestEntries, ADMITTED + 1, ADMITTED)
            ))),
            0,
            0
        )
    );
    // One left pays for the lookup, however many entries its leaves hold. No
    // page of this world carries that generation, so it verifies nothing.
    let (outcome, remaining, reads) = collect("inline-lookup-one-left", ADMITTED - 1);
    assert_eq!((outcome, remaining), (Err(HistoricalFailure::Invalid), 0));
    assert!(reads > 0, "the lookup read the segment tree");
}
