//! Atomic preflight for every alpha/Last Resort miss in one mounted work item.

use std::collections::{HashMap, HashSet};

use sha2::Digest;

use worth_ui_host_contract::UiCountedSha256;
use worth_ui_host_contract::{
    UiGlyphRasterAttribution, UiGlyphRasterDemandIdentity, UiGlyphRasterKey, UiGlyphRasterLane,
    UiQualifiedTextLayoutIdentity,
};

use super::alpha_admission::is_alpha_source;
use super::capacity::{MAX_BATCH_RECORDS, MAX_RASTER_EDGE, MAX_STAGED_BYTES};
use super::demand::UiGlyphRasterDemandBatch;
use super::denial::UiGlyphRasterizationDenial;
use super::qualified_raster_admission::{
    candidate_for_record, predicted_outline_extent, validate_demand,
};
use crate::UiQualifiedTextLayout;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct UiAlphaRasterTransactionAdmission {
    identity: [u8; 32],
    demand_batches: u32,
    unique_records: u32,
    predicted_bytes: u64,
    key_probes: u32,
    validation_checks: u32,
    provenance_checks: u32,
    admitted_keys: Box<[UiGlyphRasterKey]>,
    expected_batches: Box<[UiAlphaRasterBatchExpectation]>,
    expected_attributions: Box<[HashMap<UiGlyphRasterKey, UiGlyphRasterAttribution>]>,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(super) struct UiAlphaRasterBatchExpectation {
    pub(super) demand: UiGlyphRasterDemandIdentity,
    pub(super) layout: UiQualifiedTextLayoutIdentity,
    pub(super) scale: super::demand::UiGlyphRasterScale,
    pub(super) lane: UiGlyphRasterLane,
}

impl UiAlphaRasterTransactionAdmission {
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

    pub(super) fn expected_batch(&self, index: usize) -> Option<UiAlphaRasterBatchExpectation> {
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

/// Proves the complete mounted transaction fits before any outline evaluation
/// or raster-pixel allocation begins. Equal keys across layouts are one miss.
pub fn admit_alpha_outline_transaction(
    batches: &[(&UiQualifiedTextLayout, &UiGlyphRasterDemandBatch)],
) -> Result<UiAlphaRasterTransactionAdmission, UiGlyphRasterizationDenial> {
    let mut capacity = TransactionCapacity::default();
    let mut validation_checks = 0_u32;
    let mut provenance_checks = 0_u32;
    let mut expected_batches = Vec::with_capacity(batches.len());
    let mut expected_attributions = Vec::with_capacity(batches.len());
    for &(layout, demand) in batches {
        validate_demand(layout, demand)?;
        expected_batches.push(UiAlphaRasterBatchExpectation {
            demand: demand.identity(),
            layout: layout.identity(),
            scale: demand.scale(),
            lane: demand.lane(),
        });
        validation_checks = validation_checks
            .saturating_add(u32::try_from(demand.records().len()).unwrap_or(u32::MAX));
        provenance_checks = provenance_checks
            .saturating_add(u32::try_from(demand.provenance().len()).unwrap_or(u32::MAX));
        let mut batch_attributions = HashMap::new();
        for (index, record) in demand.records().iter().copied().enumerate() {
            if !is_alpha_source(record.key()) {
                continue;
            }
            worth_ui_host_contract::record_presentation_map_inserts(1);
            batch_attributions
                .entry(record.key())
                .or_insert(record.attribution());
            capacity.key_probes = capacity.key_probes.saturating_add(1);
            if capacity.keys.contains(&record.key()) {
                continue;
            }
            let candidate = candidate_for_record(layout, demand, index, record)?;
            let (width, height) = predicted_outline_extent(&candidate, record.key())
                .ok_or(UiGlyphRasterizationDenial::ExtentExceeded)?;
            if width > MAX_RASTER_EDGE || height > MAX_RASTER_EDGE {
                return Err(UiGlyphRasterizationDenial::ExtentExceeded);
            }
            capacity.admit(record.key(), u64::from(width) * u64::from(height))?;
        }
        expected_attributions.push(batch_attributions);
    }
    let admitted_keys = capacity.keys.iter().copied().collect::<Box<[_]>>();
    Ok(UiAlphaRasterTransactionAdmission {
        identity: transaction_identity(batches),
        demand_batches: u32::try_from(batches.len()).unwrap_or(u32::MAX),
        unique_records: u32::try_from(capacity.keys.len()).unwrap_or(u32::MAX),
        predicted_bytes: capacity.predicted_bytes,
        key_probes: capacity.key_probes,
        validation_checks,
        provenance_checks,
        admitted_keys,
        expected_batches: expected_batches.into_boxed_slice(),
        expected_attributions: expected_attributions.into_boxed_slice(),
    })
}

pub(super) fn transaction_identity(
    batches: &[(&UiQualifiedTextLayout, &UiGlyphRasterDemandBatch)],
) -> [u8; 32] {
    let mut digest = UiCountedSha256::new();
    digest.update(b"worth-ui-alpha-raster-transaction-v1\0");
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

#[derive(Default)]
struct TransactionCapacity {
    keys: HashSet<UiGlyphRasterKey>,
    predicted_bytes: u64,
    key_probes: u32,
}

impl TransactionCapacity {
    fn admit(
        &mut self,
        key: UiGlyphRasterKey,
        predicted_bytes: u64,
    ) -> Result<(), UiGlyphRasterizationDenial> {
        if self.keys.len() == MAX_BATCH_RECORDS {
            return Err(UiGlyphRasterizationDenial::BatchCapacityExceeded);
        }
        let next_bytes = self
            .predicted_bytes
            .checked_add(predicted_bytes)
            .ok_or(UiGlyphRasterizationDenial::StagedByteCapacityExceeded)?;
        if next_bytes > MAX_STAGED_BYTES {
            return Err(UiGlyphRasterizationDenial::StagedByteCapacityExceeded);
        }
        worth_ui_host_contract::record_presentation_map_inserts(1);
        self.keys.insert(key);
        self.predicted_bytes = next_bytes;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use worth_ui_host_contract::{
        UiFontCollectionGeneration, UiFontCollectionLineageIdentity, UiGlyphRasterFractionalOrigin,
        UiGlyphRasterKeyInput, UiGlyphRasterPalette, UiGlyphRasterSize, UiGlyphRasterSource,
        UiGlyphVariationCoordinates, UiQualifiedFontFaceIdentity, UiTextProfileGeneration,
    };

    #[test]
    fn two_lawful_halves_are_denied_when_the_transaction_exceeds_capacity() {
        for start in [0, MAX_BATCH_RECORDS / 2] {
            let mut half = TransactionCapacity::default();
            for offset in 0..MAX_BATCH_RECORDS / 2 {
                half.admit(key((start + offset) as u32), 1).unwrap();
            }
            assert_eq!(half.keys.len(), MAX_BATCH_RECORDS / 2);
        }
        let mut capacity = TransactionCapacity::default();
        for glyph_id in 0..MAX_BATCH_RECORDS {
            capacity.admit(key(glyph_id as u32), 1).unwrap();
        }
        assert_eq!(capacity.keys.len(), MAX_BATCH_RECORDS);
        assert_eq!(
            capacity.admit(key(MAX_BATCH_RECORDS as u32), 1),
            Err(UiGlyphRasterizationDenial::BatchCapacityExceeded)
        );
        assert_eq!(capacity.keys.len(), MAX_BATCH_RECORDS);
        assert_eq!(capacity.predicted_bytes, MAX_BATCH_RECORDS as u64);
    }

    #[test]
    fn an_omitted_admitted_key_cannot_complete_the_transaction() {
        let admitted = [key(1), key(2)];
        let admission = UiAlphaRasterTransactionAdmission {
            identity: [3; 32],
            demand_batches: 1,
            unique_records: 2,
            predicted_bytes: 2,
            key_probes: 2,
            validation_checks: 2,
            provenance_checks: 2,
            admitted_keys: Box::new(admitted),
            expected_batches: Box::new([]),
            expected_attributions: Box::new([]),
        };
        assert_eq!(
            super::super::alpha_transaction_completion::validate_produced_keys(
                &admission,
                [(0, admitted[0], attribution(), 1)],
            ),
            Err(UiGlyphRasterizationDenial::TransactionOutputMismatch)
        );
    }

    fn key(glyph_id: u32) -> UiGlyphRasterKey {
        UiGlyphRasterKey::from_text_mechanics(UiGlyphRasterKeyInput {
            font_collection: UiFontCollectionGeneration::new(1).unwrap(),
            font_collection_lineage: UiFontCollectionLineageIdentity::from_text_mechanics([1; 32]),
            profile: UiTextProfileGeneration::new(1).unwrap(),
            face: UiQualifiedFontFaceIdentity::from_text_mechanics([2; 32], 0),
            glyph_id,
            variations: UiGlyphVariationCoordinates::empty(),
            palette: UiGlyphRasterPalette::new(0),
            size: UiGlyphRasterSize::from_millipoints(14_000).unwrap(),
            source: UiGlyphRasterSource::AlphaOutline,
            dpi_milli: 1_000,
            origin: UiGlyphRasterFractionalOrigin::from_sixty_fourths(0, 0),
        })
        .unwrap()
    }

    fn attribution() -> UiGlyphRasterAttribution {
        UiGlyphRasterAttribution::from_text_mechanics(
            UiQualifiedTextLayoutIdentity::from_text_mechanics([7; 32]),
            worth_ui_host_contract::UiTextOriginalRange::new(0, 1).unwrap(),
        )
    }
}
