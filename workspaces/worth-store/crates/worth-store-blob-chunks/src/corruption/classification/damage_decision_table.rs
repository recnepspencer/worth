use super::damage_case::BlobDamageCase;
use crate::corruption::types::{BlobCorruptionDetectionSource, BlobCorruptionPlacementClass};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum LocalizationEligibilityCase {
    FrontierMatched,
    OrdinalNotInFrontier,
    GenerationFrontierMismatch,
}

pub(crate) fn classify_damage_case_from_detection_context(
    source: BlobCorruptionDetectionSource,
    placement: BlobCorruptionPlacementClass,
) -> BlobDamageCase {
    match (source, placement) {
        (
            BlobCorruptionDetectionSource::ImportReadmission,
            BlobCorruptionPlacementClass::ImportStaging,
        ) => BlobDamageCase::CrossScopeImport,
        (BlobCorruptionDetectionSource::ColdFetch, _) => BlobDamageCase::MissingChunk,
        (BlobCorruptionDetectionSource::CapsuleMaterialization, _) => {
            BlobDamageCase::AuthenticityFailure
        }
        _ => BlobDamageCase::ChecksumMismatch,
    }
}

pub(crate) const fn classify_streaming_read_damage_from_checksum_match(
    checksums_match: bool,
) -> Option<BlobDamageCase> {
    if checksums_match {
        None
    } else {
        Some(BlobDamageCase::ChecksumMismatch)
    }
}

pub(crate) const fn classify_localization_eligibility_from_frontier_match(
    generation_matches_frontier: bool,
    ordinal_in_frontier: bool,
) -> LocalizationEligibilityCase {
    if !generation_matches_frontier {
        LocalizationEligibilityCase::GenerationFrontierMismatch
    } else if !ordinal_in_frontier {
        LocalizationEligibilityCase::OrdinalNotInFrontier
    } else {
        LocalizationEligibilityCase::FrontierMatched
    }
}

pub(crate) const fn damage_case_for_localization_denial(
    case: LocalizationEligibilityCase,
) -> BlobDamageCase {
    match case {
        LocalizationEligibilityCase::GenerationFrontierMismatch => BlobDamageCase::StaleGeneration,
        LocalizationEligibilityCase::OrdinalNotInFrontier => BlobDamageCase::MissingChunk,
        LocalizationEligibilityCase::FrontierMatched => BlobDamageCase::ChecksumMismatch,
    }
}
