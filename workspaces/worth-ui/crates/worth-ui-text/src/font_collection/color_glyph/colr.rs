use std::collections::{BTreeMap, BTreeSet};

use harfrust::FontRef;
use read_fonts::{
    tables::colr::Colr,
    traversal::{FieldType, SomeArray, SomeTable},
    TableProvider,
};

use crate::font_collection::UiFontCollectionAdmissionDenial;

use super::{boundedness, malformed, traversal::validate_table};

const MAX_PAINT_GRAPH_DEPTH: u8 = 64;

pub(super) fn validate(
    font: &FontRef<'_>,
    glyph_count: u16,
) -> Result<Box<[u16]>, UiFontCollectionAdmissionDenial> {
    let colr = font.colr().map_err(|_| malformed())?;
    let cpal = font.cpal().map_err(|_| malformed())?;
    if !matches!(colr.version(), 0 | 1) || !matches!(cpal.version(), 0 | 1) {
        return Err(malformed());
    }
    let palette_entries = cpal.num_palette_entries();
    let palettes = cpal.num_palettes();
    let color_records = cpal.num_color_records();
    if palette_entries == 0 || palettes == 0 || color_records == 0 {
        return Err(malformed());
    }
    validate_table(&cpal, 0, glyph_count, palette_entries, None)?;
    let records = cpal
        .color_records_array()
        .ok_or_else(malformed)?
        .map_err(|_| malformed())?;
    if records.len() != usize::from(color_records)
        || cpal.color_record_indices().len() != usize::from(palettes)
        || cpal
            .color_record_indices()
            .iter()
            .any(|first| usize::from(first.get()) + usize::from(palette_entries) > records.len())
    {
        return Err(malformed());
    }
    let mut glyphs = validate_v0(&colr, glyph_count, palette_entries)?;
    validate_table(&colr, 0, glyph_count, palette_entries, Some(&colr))?;
    let v1_glyphs = if colr.version() == 1 {
        validate_v1(&colr, glyph_count, palette_entries)?
    } else {
        BTreeSet::new()
    };
    boundedness::validate(font, v1_glyphs.iter().copied())?;
    glyphs.extend(v1_glyphs);
    if glyphs.is_empty() {
        Err(malformed())
    } else {
        Ok(glyphs.into_iter().collect())
    }
}

fn validate_v0(
    colr: &Colr<'_>,
    glyph_count: u16,
    palette_entries: u16,
) -> Result<BTreeSet<u16>, UiFontCollectionAdmissionDenial> {
    let base_records = colr
        .base_glyph_records()
        .transpose()
        .map_err(|_| malformed())?;
    let layers = colr.layer_records().transpose().map_err(|_| malformed())?;
    let expected_bases = usize::from(colr.num_base_glyph_records());
    let expected_layers = usize::from(colr.num_layer_records());
    if base_records.map_or(0, <[_]>::len) != expected_bases
        || layers.map_or(0, <[_]>::len) != expected_layers
    {
        return Err(malformed());
    }
    let base_records = base_records.unwrap_or_default();
    let layers = layers.unwrap_or_default();
    let mut prior = None;
    let mut glyphs = BTreeSet::new();
    for record in base_records {
        let glyph = record.glyph_id().to_u16();
        if glyph >= glyph_count || prior.is_some_and(|value| value >= glyph) {
            return Err(malformed());
        }
        prior = Some(glyph);
        glyphs.insert(glyph);
        let end = usize::from(record.first_layer_index())
            .checked_add(usize::from(record.num_layers()))
            .ok_or_else(malformed)?;
        if end > layers.len() {
            return Err(malformed());
        }
    }
    for layer in layers {
        if layer.glyph_id().to_u16() >= glyph_count
            || !valid_palette(layer.palette_index(), palette_entries)
        {
            return Err(malformed());
        }
    }
    Ok(glyphs)
}

fn validate_v1(
    colr: &Colr<'_>,
    glyph_count: u16,
    palette_entries: u16,
) -> Result<BTreeSet<u16>, UiFontCollectionAdmissionDenial> {
    let Some(list) = colr.base_glyph_list() else {
        if colr.layer_list().is_some() || colr.clip_list().is_some() {
            return Err(malformed());
        }
        return Ok(BTreeSet::new());
    };
    let list = list.map_err(|_| malformed())?;
    let mut graph = BTreeMap::<u16, BTreeSet<u16>>::new();
    let mut prior = None;
    for record in list.base_glyph_paint_records() {
        let glyph = record.glyph_id().to_u16();
        if glyph >= glyph_count || prior.is_some_and(|value| value >= glyph) {
            return Err(malformed());
        }
        prior = Some(glyph);
        let paint = record.paint(list.offset_data()).map_err(|_| malformed())?;
        validate_table(&paint, 0, glyph_count, palette_entries, Some(colr))?;
        let mut references = BTreeSet::new();
        collect_references(&paint, colr, 0, &mut references)?;
        graph.insert(glyph, references);
    }
    for references in graph.values() {
        if references.iter().any(|glyph| !graph.contains_key(glyph)) {
            return Err(malformed());
        }
    }
    let mut settled = BTreeSet::new();
    for glyph in graph.keys().copied() {
        reject_cycle(glyph, &graph, &mut BTreeSet::new(), &mut settled)?;
    }
    Ok(graph.into_keys().collect())
}

fn reject_cycle(
    glyph: u16,
    graph: &BTreeMap<u16, BTreeSet<u16>>,
    active: &mut BTreeSet<u16>,
    settled: &mut BTreeSet<u16>,
) -> Result<(), UiFontCollectionAdmissionDenial> {
    if settled.contains(&glyph) {
        return Ok(());
    }
    if !active.insert(glyph) {
        return Err(malformed());
    }
    for successor in &graph[&glyph] {
        reject_cycle(*successor, graph, active, settled)?;
    }
    active.remove(&glyph);
    settled.insert(glyph);
    Ok(())
}

fn collect_references(
    table: &dyn SomeTable<'_>,
    colr: &Colr<'_>,
    depth: u8,
    references: &mut BTreeSet<u16>,
) -> Result<(), UiFontCollectionAdmissionDenial> {
    if depth >= MAX_PAINT_GRAPH_DEPTH {
        return Err(malformed());
    }
    let mut first_layer = None;
    let mut layer_count = None;
    let mut index = 0;
    while let Some(field) = table.get_field(index) {
        index += 1;
        if table.type_name() == "PaintColrGlyph"
            && field.name == "glyph_id"
            && let FieldType::GlyphId16(glyph) = &field.value
        {
            references.insert(glyph.to_u16());
        }
        match (field.name, &field.value) {
            ("first_layer_index", FieldType::U32(value)) => first_layer = Some(*value),
            ("num_layers", FieldType::U8(value)) => layer_count = Some(*value),
            _ => {}
        }
        collect_field(field.value, colr, depth + 1, references)?;
    }
    if table.type_name() == "PaintColrLayers" {
        let start = first_layer.ok_or_else(malformed)?;
        let count = u32::from(layer_count.ok_or_else(malformed)?);
        for index in start..start.checked_add(count).ok_or_else(malformed)? {
            let (paint, _) = colr.v1_layer(index as usize).map_err(|_| malformed())?;
            collect_references(&paint, colr, depth + 1, references)?;
        }
    }
    Ok(())
}

fn collect_field(
    field: FieldType<'_>,
    colr: &Colr<'_>,
    depth: u8,
    references: &mut BTreeSet<u16>,
) -> Result<(), UiFontCollectionAdmissionDenial> {
    match field {
        FieldType::ResolvedOffset(offset) => collect_references(
            offset.target.map_err(|_| malformed())?.as_ref(),
            colr,
            depth,
            references,
        ),
        FieldType::Record(record) => collect_references(&record, colr, depth, references),
        FieldType::ArrayOffset(array) => collect_array(
            array.target.map_err(|_| malformed())?.as_ref(),
            colr,
            depth,
            references,
        ),
        FieldType::Array(array) => collect_array(array.as_ref(), colr, depth, references),
        _ => Ok(()),
    }
}

fn collect_array(
    array: &dyn SomeArray<'_>,
    colr: &Colr<'_>,
    depth: u8,
    references: &mut BTreeSet<u16>,
) -> Result<(), UiFontCollectionAdmissionDenial> {
    for index in 0..array.len() {
        if let Some(field) = array.get(index) {
            collect_field(field, colr, depth, references)?;
        }
    }
    Ok(())
}

pub(super) const fn valid_palette(index: u16, palette_entries: u16) -> bool {
    index < palette_entries
}
