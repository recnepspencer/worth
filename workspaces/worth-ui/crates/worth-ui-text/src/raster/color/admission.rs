//! Effect-free aggregate admission for qualified intrinsic-color misses.

use std::collections::{HashMap, HashSet};

use sha2::Digest;

use worth_ui_host_contract::UiCountedSha256;
use worth_ui_host_contract::{
    UiGlyphRasterAttribution, UiGlyphRasterDemandIdentity, UiGlyphRasterKey, UiGlyphRasterLane,
    UiQualifiedTextLayoutIdentity,
};

use super::super::capacity::{MAX_BATCH_RECORDS, MAX_STAGED_BYTES};
use super::super::demand::UiGlyphRasterDemandBatch;
use super::super::denial::UiGlyphRasterizationDenial;
use super::super::qualified_raster_admission::{
    admit_extent, candidate_for_record, validate_demand,
};
use super::image::predicted_geometry;
use crate::UiQualifiedTextLayout;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct UiColorRasterAdmission {
    unique_records: usize,
    predicted_bytes: u64,
    validation_checks: u32,
    provenance_checks: u32,
}

impl UiColorRasterAdmission {
    pub const fn unique_records(self) -> usize {
        self.unique_records
    }

    pub const fn predicted_bytes(self) -> u64 {
        self.predicted_bytes
    }

    pub const fn validation_checks(self) -> u32 {
        self.validation_checks
    }

    pub const fn provenance_checks(self) -> u32 {
        self.provenance_checks
    }
}

pub fn admit_intrinsic_color(
    layout: &UiQualifiedTextLayout,
    demand: &UiGlyphRasterDemandBatch,
) -> Result<UiColorRasterAdmission, UiGlyphRasterizationDenial> {
    validate_demand(layout, demand)?;
    let mut keys = HashSet::with_capacity(demand.records().len());
    let mut predicted_bytes = 0_u64;
    for (index, record) in demand.records().iter().copied().enumerate() {
        if !is_color_source(record.key()) || !keys.insert(record.key()) {
            continue;
        }
        if keys.len() > MAX_BATCH_RECORDS {
            return Err(UiGlyphRasterizationDenial::BatchCapacityExceeded);
        }
        let candidate = candidate_for_record(layout, demand, index, record)?;
        validate_color_source(layout, record.key())?;
        validate_color_palette(layout, record.key())?;
        let geometry = predicted_geometry(layout, &candidate)?;
        admit_extent(geometry.width, geometry.height, &mut predicted_bytes)?;
    }
    Ok(UiColorRasterAdmission {
        unique_records: keys.len(),
        predicted_bytes,
        validation_checks: u32::try_from(demand.records().len()).unwrap_or(u32::MAX),
        provenance_checks: u32::try_from(demand.provenance().len()).unwrap_or(u32::MAX),
    })
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct UiColorRasterTransactionAdmission {
    identity: [u8; 32],
    demand_batches: u32,
    unique_records: u32,
    predicted_bytes: u64,
    key_probes: u32,
    validation_checks: u32,
    provenance_checks: u32,
    admitted_keys: Box<[UiGlyphRasterKey]>,
    expected_batches: Box<[UiColorRasterBatchExpectation]>,
    expected_attributions: Box<[HashMap<UiGlyphRasterKey, UiGlyphRasterAttribution>]>,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(super) struct UiColorRasterBatchExpectation {
    pub(super) demand: UiGlyphRasterDemandIdentity,
    pub(super) layout: UiQualifiedTextLayoutIdentity,
    pub(super) scale: super::super::demand::UiGlyphRasterScale,
    pub(super) lane: UiGlyphRasterLane,
}

impl UiColorRasterTransactionAdmission {
    pub(crate) const fn identity(&self) -> [u8; 32] {
        self.identity
    }

    pub const fn demand_batches(&self) -> u32 {
        self.demand_batches
    }

    pub const fn unique_records(&self) -> u32 {
        self.unique_records
    }

    pub const fn predicted_bytes(&self) -> u64 {
        self.predicted_bytes
    }

    pub const fn key_probes(&self) -> u32 {
        self.key_probes
    }

    pub const fn validation_checks(&self) -> u32 {
        self.validation_checks
    }

    pub const fn provenance_checks(&self) -> u32 {
        self.provenance_checks
    }

    pub(super) fn admits_key(&self, key: UiGlyphRasterKey) -> bool {
        self.admitted_keys.contains(&key)
    }

    pub(super) fn admitted_keys(&self) -> &[UiGlyphRasterKey] {
        &self.admitted_keys
    }

    pub(super) fn expected_batch(&self, index: usize) -> Option<UiColorRasterBatchExpectation> {
        self.expected_batches.get(index).copied()
    }

    pub(super) fn expected_attribution(
        &self,
        batch_index: usize,
        key: UiGlyphRasterKey,
    ) -> Option<UiGlyphRasterAttribution> {
        self.expected_attributions
            .get(batch_index)?
            .get(&key)
            .copied()
    }
}

pub fn admit_intrinsic_color_transaction(
    batches: &[(&UiQualifiedTextLayout, &UiGlyphRasterDemandBatch)],
) -> Result<UiColorRasterTransactionAdmission, UiGlyphRasterizationDenial> {
    let mut capacity = ColorTransactionCapacity::default();
    let mut validation_checks = 0_u32;
    let mut provenance_checks = 0_u32;
    let mut expected_batches = Vec::with_capacity(batches.len());
    let mut expected_attributions = Vec::new();
    for &(layout, demand) in batches {
        validate_demand(layout, demand)?;
        expected_batches.push(UiColorRasterBatchExpectation {
            demand: demand.identity(),
            layout: layout.identity(),
            scale: demand.scale(),
            lane: demand.lane(),
        });
        validation_checks = validation_checks
            .saturating_add(u32::try_from(demand.records().len()).unwrap_or(u32::MAX));
        provenance_checks = provenance_checks
            .saturating_add(u32::try_from(demand.provenance().len()).unwrap_or(u32::MAX));
        let mut batch_keys = HashSet::new();
        let mut batch_attributions = HashMap::new();
        for (index, record) in demand.records().iter().copied().enumerate() {
            if !is_color_source(record.key()) {
                continue;
            }
            if batch_keys.insert(record.key()) {
                batch_attributions.insert(record.key(), record.attribution());
            }
            capacity.key_probes = capacity.key_probes.saturating_add(1);
            if capacity.keys.contains(&record.key()) {
                continue;
            }
            let candidate = candidate_for_record(layout, demand, index, record)?;
            validate_color_source(layout, record.key())?;
            validate_color_palette(layout, record.key())?;
            let geometry = predicted_geometry(layout, &candidate)?;
            capacity.admit(record.key(), geometry.width, geometry.height)?;
        }
        expected_attributions.push(batch_attributions);
    }
    Ok(UiColorRasterTransactionAdmission {
        identity: transaction_identity(batches),
        demand_batches: u32::try_from(batches.len()).unwrap_or(u32::MAX),
        unique_records: u32::try_from(capacity.keys.len()).unwrap_or(u32::MAX),
        predicted_bytes: capacity.predicted_bytes,
        key_probes: capacity.key_probes,
        validation_checks,
        provenance_checks,
        admitted_keys: capacity.keys.into_iter().collect(),
        expected_batches: expected_batches.into_boxed_slice(),
        expected_attributions: expected_attributions.into_boxed_slice(),
    })
}

pub(super) fn transaction_identity(
    batches: &[(&UiQualifiedTextLayout, &UiGlyphRasterDemandBatch)],
) -> [u8; 32] {
    let mut digest = UiCountedSha256::new();
    digest.update(b"worth-ui-color-raster-transaction-v1\0");
    digest.update(
        u64::try_from(batches.len())
            .unwrap_or(u64::MAX)
            .to_le_bytes(),
    );
    for &(layout, demand) in batches {
        digest.update(layout.identity().digest());
        digest.update(demand.identity().digest());
        digest.update(demand.scale().dpi_milli().to_le_bytes());
        digest.update(demand.scale().text_scale_generation().get().to_le_bytes());
        digest.update([match demand.lane() {
            worth_ui_host_contract::UiGlyphRasterLane::Ordinary => 0,
            worth_ui_host_contract::UiGlyphRasterLane::Reconstruction => 1,
        }]);
    }
    digest.finalize().into()
}

pub(super) fn is_color_source(key: UiGlyphRasterKey) -> bool {
    matches!(
        key.source(),
        worth_ui_host_contract::UiGlyphRasterSource::ColorOutline
            | worth_ui_host_contract::UiGlyphRasterSource::ColorBitmap
    )
}

pub(super) fn validate_color_source(
    layout: &UiQualifiedTextLayout,
    key: UiGlyphRasterKey,
) -> Result<(), UiGlyphRasterizationDenial> {
    let resource = layout
        .artifact()
        .face_resource(key.face())
        .ok_or(UiGlyphRasterizationDenial::InvalidFaceResource)?;
    let matches = matches!(
        (resource.color_source(key.glyph_id()), key.source()),
        (
            Some(crate::layout_artifact::UiQualifiedTextColorSource::Outline),
            worth_ui_host_contract::UiGlyphRasterSource::ColorOutline
        ) | (
            Some(crate::layout_artifact::UiQualifiedTextColorSource::Bitmap),
            worth_ui_host_contract::UiGlyphRasterSource::ColorBitmap
        )
    );
    matches
        .then_some(())
        .ok_or(UiGlyphRasterizationDenial::UnsupportedColorSource)
}

pub(super) fn validate_color_palette(
    layout: &UiQualifiedTextLayout,
    key: UiGlyphRasterKey,
) -> Result<(), UiGlyphRasterizationDenial> {
    if key.source() != worth_ui_host_contract::UiGlyphRasterSource::ColorOutline {
        return Ok(());
    }
    let resource = layout
        .artifact()
        .face_resource(key.face())
        .ok_or(UiGlyphRasterizationDenial::InvalidFaceResource)?;
    let face = swash::FontRef::from_index(
        resource.bytes(),
        usize::try_from(key.face().face_index())
            .map_err(|_| UiGlyphRasterizationDenial::InvalidFaceResource)?,
    )
    .ok_or(UiGlyphRasterizationDenial::InvalidFaceResource)?;
    face.color_palettes()
        .nth(usize::from(key.palette().index()))
        .map(|_| ())
        .ok_or(UiGlyphRasterizationDenial::InvalidColorPalette)
}

#[derive(Default)]
struct ColorTransactionCapacity {
    keys: HashSet<UiGlyphRasterKey>,
    predicted_bytes: u64,
    key_probes: u32,
}

impl ColorTransactionCapacity {
    fn admit(
        &mut self,
        key: UiGlyphRasterKey,
        width: u32,
        height: u32,
    ) -> Result<(), UiGlyphRasterizationDenial> {
        if self.keys.len() == MAX_BATCH_RECORDS {
            return Err(UiGlyphRasterizationDenial::BatchCapacityExceeded);
        }
        let mut next = self.predicted_bytes;
        admit_extent(width, height, &mut next)?;
        if next > MAX_STAGED_BYTES {
            return Err(UiGlyphRasterizationDenial::StagedByteCapacityExceeded);
        }
        self.keys.insert(key);
        self.predicted_bytes = next;
        Ok(())
    }
}

#[cfg(test)]
#[path = "admission_tests.rs"]
mod tests;
