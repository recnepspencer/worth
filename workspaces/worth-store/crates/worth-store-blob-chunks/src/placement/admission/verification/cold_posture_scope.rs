use crate::placement::admission::{
    basis::BlobPlacementReachabilityBasis, BlobPlacementAdmissionDenial,
    BlobPlacementCounterSnapshot, BlobPlacementIntent,
};

pub(crate) fn verify_cold_posture_scope(
    basis: &BlobPlacementReachabilityBasis,
    intent: &BlobPlacementIntent<'_>,
) -> Result<(), BlobPlacementAdmissionDenial> {
    if let Some(posture) = intent.cold_posture()
        && !basis.admits_cold_posture(posture)
    {
        return Err(BlobPlacementAdmissionDenial::ColdPostureScopeMismatch {
            counters: BlobPlacementCounterSnapshot::for_class(intent.class()),
        });
    }
    Ok(())
}
