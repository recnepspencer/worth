//! Host-contract demand translation and exact transaction identity admission.

use sha2::Digest;

use worth_ui_host_contract::UiCountedSha256;
use worth_ui_host_contract::{
    UiGlyphRasterDemandBatchView, UiGlyphRasterDemandIdentity, UiGlyphRasterLane,
};

use super::admission::normalize_demands;
use super::key::canonical_raster_key_bytes;
use super::ownership::AtlasCore;
use super::pinning::validate_pin_transition;
use super::recovery::UiNativeTextAtlasDenial;
use super::transaction::UiNativeTextAtlasPinTransition;
use super::UiNativeTextAtlasDemand;

pub(super) fn translate_demands(
    demands: &[UiGlyphRasterDemandBatchView<'_>],
) -> Result<(Vec<UiNativeTextAtlasDemand>, UiGlyphRasterDemandIdentity), UiNativeTextAtlasDenial> {
    let identity = combined_identity(demands);
    let mut translated = Vec::new();
    for demand in demands {
        translated.extend(translate_one(*demand, identity)?);
    }
    Ok((translated, identity))
}

pub(super) fn validate_inputs(
    core: &AtlasCore,
    demands: &[UiNativeTextAtlasDemand],
    transition: &UiNativeTextAtlasPinTransition,
) -> Result<(Vec<UiNativeTextAtlasDemand>, UiGlyphRasterDemandIdentity), UiNativeTextAtlasDenial> {
    if core.reservation.is_some() {
        return Err(UiNativeTextAtlasDenial::ReservationConflict);
    }
    let normalized = normalize_demands(demands)?;
    let identity = normalized
        .first()
        .map(|demand| demand.identity())
        .unwrap_or_else(|| UiGlyphRasterDemandIdentity::from_text_mechanics([0; 32]));
    validate_pin_transition(core, transition)?;
    Ok((normalized, identity))
}

fn translate_one(
    demand: UiGlyphRasterDemandBatchView<'_>,
    transaction_identity: UiGlyphRasterDemandIdentity,
) -> Result<Vec<UiNativeTextAtlasDemand>, UiNativeTextAtlasDenial> {
    demand
        .records()
        .iter()
        .copied()
        .map(|record| {
            if record.attribution().layout() != demand.layout_identity()
                || record.key().dpi_milli() != demand.dpi_milli()
            {
                return Err(UiNativeTextAtlasDenial::StaleDemand);
            }
            Ok(UiNativeTextAtlasDemand::from_host_contract(
                transaction_identity,
                demand,
                record,
            ))
        })
        .collect()
}

fn combined_identity(demands: &[UiGlyphRasterDemandBatchView<'_>]) -> UiGlyphRasterDemandIdentity {
    let mut digest = UiCountedSha256::new();
    digest.update(b"worth-ui-native-atlas-transaction-v1\0");
    digest.update(
        u64::try_from(demands.len())
            .unwrap_or(u64::MAX)
            .to_le_bytes(),
    );
    for demand in demands {
        digest.update(demand.identity().digest());
        digest.update(demand.layout_identity().digest());
        digest.update(demand.dpi_milli().to_le_bytes());
        digest.update(demand.text_scale_generation().get().to_le_bytes());
        digest.update([match demand.lane() {
            UiGlyphRasterLane::Ordinary => 0,
            UiGlyphRasterLane::Reconstruction => 1,
        }]);
        digest.update(
            u64::try_from(demand.records().len())
                .unwrap_or(u64::MAX)
                .to_le_bytes(),
        );
        for record in demand.records() {
            digest.update(canonical_raster_key_bytes(record.key()));
            digest.update(record.attribution().layout().digest());
            digest.update(record.attribution().original_range().start().to_le_bytes());
            digest.update(record.attribution().original_range().end().to_le_bytes());
            digest.update(record.extent().width().to_le_bytes());
            digest.update(record.extent().height().to_le_bytes());
            digest.update(record.staged_bytes().to_le_bytes());
        }
    }
    UiGlyphRasterDemandIdentity::from_text_mechanics(digest.finalize().into())
}
