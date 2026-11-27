//! Directory partition and history consolidation conformance trace builders
//! for `StoreM4.ChunkedCoverage`.
use super::super::super::canonical_read::{read_chunk_canonical, ChunkResidency, SlotSelection};
use super::super::super::chunk_directory::RegionChunkDirectory;
use super::super::super::chunk_images::{ChunkHistoryImage, HistoryLayer, HistoryRow};
use super::super::super::consolidation::{
    consolidate_history_chunk, flatten_history_image, verify_bound_chunk,
};
use super::super::super::layout::ChunkLayout;
use super::super::super::locator_binding::ChunkLocatorBinding;
use super::super::super::sealing::{seal_chunk_image, ChunkSealRequest};
use super::super::super::semantic_entry::ChunkSemanticEntry;
use super::super::super::slot_projection::{AdjacencyEdge, CanonicalChunkVisitor, SlotVersion};
use super::super::super::slot_range::{fixed_partition, SlotDomain};
use super::super::member_store::{
    accountant, edge, seal, version, MemberStore, SlotFixture, LAYOUT,
};
use super::{rows_json, u64_list_json, Trace};
use crate::storage::residency::working_set::ResidencyPoolKind;

fn domain_tag(domain: SlotDomain) -> u8 {
    match domain {
        SlotDomain::Entity => 0,
        SlotDomain::Relation => 1,
    }
}

fn version_marker(v: &SlotVersion) -> u64 {
    u64::from_be_bytes(v.payload[..8].try_into().expect("8-byte marker prefix"))
}

fn layers_json(image: &ChunkHistoryImage) -> String {
    let layers: Vec<String> = image
        .layers
        .iter()
        .map(|layer| {
            let rows: Vec<(u64, Vec<u64>)> = layer
                .rows
                .iter()
                .map(|r| (r.slot, r.versions.iter().map(version_marker).collect()))
                .collect();
            rows_json(&rows)
        })
        .collect();
    format!("[{}]", layers.join(","))
}

fn sealed_domain(
    store: &MemberStore,
    layout: ChunkLayout,
    domain: SlotDomain,
    total: u64,
) -> Vec<(ChunkSemanticEntry, ChunkLocatorBinding)> {
    fixed_partition(total, layout.chunk_slots)
        .map(|range| {
            let slot = SlotFixture {
                slot: range.start_slot_inclusive(),
                head: Some(version(1, range.start_slot_inclusive())),
                history: Vec::new(),
                adjacency: Vec::new(),
            };
            let sealed = seal_chunk_image(
                ChunkSealRequest {
                    layout,
                    domain,
                    domain_slots: total,
                    slot_range: range,
                    slots: Box::new(std::iter::once(slot.projection())),
                    claimed: None,
                },
                &mut &*store,
            )
            .expect("seal valid chunk");
            (sealed.semantic, sealed.binding)
        })
        .collect()
}

fn record_directory_case(
    trace: &mut Trace,
    layout: ChunkLayout,
    entity_slots: u64,
    relation_slots: u64,
    submitted: Vec<(ChunkSemanticEntry, ChunkLocatorBinding)>,
    probe_slots: &[(SlotDomain, u64)],
) {
    let sub_json: Vec<String> = submitted
        .iter()
        .map(|(s, _)| {
            format!(
                "[{},{},{}]",
                domain_tag(s.domain()),
                s.slot_range().start_slot_inclusive(),
                s.slot_range().end_slot_exclusive()
            )
        })
        .collect();
    let res = RegionChunkDirectory::admit(layout, entity_slots, relation_slots, submitted);
    let (admitted, lookups_json) = match res {
        Ok(dir) => {
            let rows: Vec<String> = probe_slots
                .iter()
                .map(|&(dom, slot)| {
                    let found = dir.chunk_of(dom, slot).map_or_else(
                        || "null".to_owned(),
                        |ch| {
                            let r = ch.semantic().slot_range();
                            format!("[{},{}]", r.start_slot_inclusive(), r.end_slot_exclusive())
                        },
                    );
                    format!("[{},{slot},{found}]", domain_tag(dom))
                })
                .collect();
            (true, format!("[{}]", rows.join(",")))
        }
        Err(_) => (false, "[]".to_owned()),
    };
    trace.push(
        "relational_chunk_directory",
        format!(
            r#""chunk_slots":{},"entity_slots":{entity_slots},"relation_slots":{relation_slots},"submitted":[{}],"admitted":{admitted},"slot_lookups":{lookups_json}"#,
            layout.chunk_slots,
            sub_json.join(","),
        ),
    );
}

pub(super) fn record_directory_cases(trace: &mut Trace) {
    let store = MemberStore::default();
    let entities = sealed_domain(&store, LAYOUT, SlotDomain::Entity, 200);
    let relations = sealed_domain(&store, LAYOUT, SlotDomain::Relation, 70);
    let all = || {
        entities
            .iter()
            .chain(&relations)
            .cloned()
            .collect::<Vec<_>>()
    };
    let probes = [
        (SlotDomain::Entity, 0),
        (SlotDomain::Entity, 63),
        (SlotDomain::Entity, 64),
        (SlotDomain::Entity, 199),
        (SlotDomain::Entity, 200),
        (SlotDomain::Relation, 0),
        (SlotDomain::Relation, 69),
        (SlotDomain::Relation, 70),
    ];
    record_directory_case(trace, LAYOUT, 200, 70, all(), &probes);
    let single = sealed_domain(&store, LAYOUT, SlotDomain::Entity, 64);
    record_directory_case(
        trace,
        LAYOUT,
        64,
        0,
        single,
        &[(SlotDomain::Entity, 0), (SlotDomain::Entity, 63)],
    );
    record_directory_case(trace, LAYOUT, 0, 0, Vec::new(), &[(SlotDomain::Entity, 0)]);

    let mut gap = all();
    gap.remove(1);
    record_directory_case(trace, LAYOUT, 200, 70, gap, &[]);
    let mut overlap = all();
    overlap.insert(1, overlap[0].clone());
    record_directory_case(trace, LAYOUT, 200, 70, overlap, &[]);
    let mut missing_tail = all();
    missing_tail.pop();
    record_directory_case(trace, LAYOUT, 200, 70, missing_tail, &[]);
    let mut unsorted = all();
    unsorted.swap(0, 1);
    record_directory_case(trace, LAYOUT, 200, 70, unsorted, &[]);
    let half = ChunkLayout {
        chunk_slots: 32,
        ..LAYOUT
    };
    let misaligned = sealed_domain(&store, half, SlotDomain::Entity, 64);
    record_directory_case(trace, LAYOUT, 64, 0, misaligned, &[]);
    record_directory_case(trace, LAYOUT, 201, 70, all(), &[]);
    let zero = ChunkLayout {
        chunk_slots: 0,
        ..LAYOUT
    };
    record_directory_case(trace, zero, 10, 0, Vec::new(), &[]);
}

#[derive(Default)]
struct CanonicalHistoryCollector {
    current_slot: u64,
    current_versions: Vec<u64>,
    rows: Vec<(u64, Vec<u64>)>,
}

impl CanonicalChunkVisitor for CanonicalHistoryCollector {
    type Error = std::convert::Infallible;
    fn slot_begin(&mut self, slot: u64, _: Option<&SlotVersion>) -> Result<(), Self::Error> {
        self.current_slot = slot;
        self.current_versions.clear();
        Ok(())
    }
    fn history_version(&mut self, version: &SlotVersion) -> Result<(), Self::Error> {
        self.current_versions.push(version_marker(version));
        Ok(())
    }
    fn adjacency_edge(&mut self, _: &AdjacencyEdge) -> Result<(), Self::Error> {
        Ok(())
    }
    fn slot_end(&mut self) -> Result<(), Self::Error> {
        self.rows.push((
            self.current_slot,
            std::mem::take(&mut self.current_versions),
        ));
        Ok(())
    }
}

fn canonical_history(
    residency: &ChunkResidency,
    store: &MemberStore,
    binding: &ChunkLocatorBinding,
) -> Vec<(u64, Vec<u64>)> {
    let mut c = CanonicalHistoryCollector::default();
    read_chunk_canonical(
        residency,
        store,
        SlotDomain::Entity,
        binding,
        SlotSelection::All,
        &mut c,
    )
    .expect("read canonical chunk");
    c.rows
}

fn split_into_two_layers(image: &ChunkHistoryImage) -> ChunkHistoryImage {
    let (mut older, mut newer) = (HistoryLayer::default(), HistoryLayer::default());
    for row in &image.layers[0].rows {
        let (left, right) = row.versions.split_at(row.versions.len() / 2);
        for (layer, vs) in [(&mut older, left), (&mut newer, right)] {
            if !vs.is_empty() {
                layer.rows.push(HistoryRow {
                    slot: row.slot,
                    versions: vs.to_vec(),
                });
            }
        }
    }
    ChunkHistoryImage {
        layers: vec![older, newer],
        ..image.clone()
    }
}

pub(super) fn record_consolidation_cases(trace: &mut Trace) {
    let store = MemberStore::default();
    let range = fixed_partition(64, LAYOUT.chunk_slots).next().unwrap();
    let fixtures: Vec<SlotFixture> = (0..4)
        .map(|i| SlotFixture {
            slot: i * 10,
            head: Some(version(1, i)),
            history: (0..if i == 2 { 110 } else { 4 + i })
                .map(|v| version(5, i * 1_000 + v))
                .collect(),
            adjacency: vec![edge(20, i)],
        })
        .collect();
    let sealed = seal(&store, SlotDomain::Entity, 64, range, &fixtures);
    let residency = ChunkResidency::new(
        &accountant(1 << 20, 1 << 20, 4096),
        ResidencyPoolKind::Historical,
    );
    let base_hist = residency
        .history_image(&store, sealed.binding.history_locator())
        .unwrap();
    let layered = split_into_two_layers(&base_hist);
    let layered_enc = layered.encode(LAYOUT.max_chunk_image_bytes).unwrap();
    store.replace(layered_enc.locator, layered_enc.bytes);
    let layered_bind = sealed.binding.with_history_locator(layered_enc.locator);
    let mut dir =
        RegionChunkDirectory::admit(LAYOUT, 64, 0, [(sealed.semantic.clone(), layered_bind)])
            .unwrap();
    let root_before = dir.root_semantic_merkle();
    let slots: Vec<u64> = fixtures.iter().map(|f| f.slot).collect();
    let canon_before = canonical_history(&residency, &store, &layered_bind);
    let inline_before: Vec<(u64, Vec<u64>)> = slots
        .iter()
        .map(|&s| (s, layered.slot_versions(s).map(version_marker).collect()))
        .collect();
    let spilled: Vec<(u64, Vec<u64>)> = canon_before
        .iter()
        .zip(&inline_before)
        .map(|((s, all), (_, inl))| {
            let k = all.len() - inl.len();
            assert_eq!(&all[k..], inl.as_slice());
            (*s, all[..k].to_vec())
        })
        .collect();

    let flat_loc =
        flatten_history_image(&residency, &store, &mut &store, LAYOUT, &layered_bind).unwrap();
    assert_ne!(flat_loc, layered_enc.locator);
    let flat_img = residency.history_image(&store, flat_loc).unwrap();
    let flat_bind = layered_bind.with_history_locator(flat_loc);
    let v_ok = verify_bound_chunk(&residency, &store, &sealed.semantic, &flat_bind).is_ok();
    let cons = consolidate_history_chunk(
        &residency,
        &store,
        &sealed.semantic,
        &layered_bind,
        flat_loc,
    );
    let c_ok = cons.is_ok();
    dir.install_consolidated(cons.unwrap()).unwrap();
    let merkle_ok = dir.root_semantic_merkle() == root_before;
    let inline_after: Vec<(u64, Vec<u64>)> = slots
        .iter()
        .map(|&s| (s, flat_img.slot_versions(s).map(version_marker).collect()))
        .collect();
    let canon_after = canonical_history(&residency, &store, &flat_bind);
    trace.push(
        "relational_chunk_consolidation",
        format!(
            r#""slots":{},"spilled":{},"layers_before":{},"layers_after":{},"inline_before":{},"inline_after":{},"canonical_before":{},"canonical_after":{},"is_flattened_candidate":true,"verify_ok":{v_ok},"consolidate_ok":{c_ok},"merkle_preserved":{merkle_ok}"#,
            u64_list_json(&slots), rows_json(&spilled), layers_json(&layered), layers_json(&flat_img),
            rows_json(&inline_before), rows_json(&inline_after), rows_json(&canon_before), rows_json(&canon_after),
        ),
    );

    let mut lossy = base_hist.flattened();
    lossy.layers[0].rows[0].versions.pop();
    let lossy_enc = lossy.encode(LAYOUT.max_chunk_image_bytes).unwrap();
    store.replace(lossy_enc.locator, lossy_enc.bytes);
    let lossy_bind = layered_bind.with_history_locator(lossy_enc.locator);
    let v_lossy = verify_bound_chunk(&residency, &store, &sealed.semantic, &lossy_bind).is_ok();
    let c_lossy = consolidate_history_chunk(
        &residency,
        &store,
        &sealed.semantic,
        &layered_bind,
        lossy_enc.locator,
    )
    .is_ok();
    let lossy_inl: Vec<(u64, Vec<u64>)> = slots
        .iter()
        .map(|&s| (s, lossy.slot_versions(s).map(version_marker).collect()))
        .collect();
    let lossy_canon = canonical_history(&residency, &store, &lossy_bind);
    trace.push(
        "relational_chunk_consolidation",
        format!(
            r#""slots":{},"spilled":{},"layers_before":{},"layers_after":{},"inline_before":{},"inline_after":{},"canonical_before":{},"canonical_after":{},"is_flattened_candidate":false,"verify_ok":{v_lossy},"consolidate_ok":{c_lossy},"merkle_preserved":true"#,
            u64_list_json(&slots), rows_json(&spilled), layers_json(&layered), layers_json(&lossy),
            rows_json(&inline_before), rows_json(&lossy_inl), rows_json(&canon_before), rows_json(&lossy_canon),
        ),
    );
}
