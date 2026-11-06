use crate::evidence_identity::WorthQueryEvidenceIdentityDerivationDenial as IdentityDerivationDenial;
use crate::evidence_identity::{
    WorthQueryEvidenceIdentity, WorthQueryEvidenceScope, WorthQueryEvidenceTag,
};
use crate::runtime::{WorthQueryReadReceipt, WorthQueryWriteReceiptInspection};

use super::inventory::CausalEvidenceFamily;
use super::observation_identity::{
    CausalEvidenceReferenceDigest, CausalObservationBasisIdentity, CausalObservationQueryIdentity,
    CausalObservationReceiptIdentity,
};
use super::receipt_types::{CausalObservationBasisPosture, QueryObservationReceiptFamily};

pub(super) fn causal_observation_receipt_evidence_identity(
    family: QueryObservationReceiptFamily,
    source_receipt: &WorthQueryEvidenceIdentity,
) -> Result<CausalObservationReceiptIdentity, IdentityDerivationDenial> {
    Ok(
        WorthQueryEvidenceIdentity::compose(WorthQueryEvidenceScope::CausalObservationReceipt)
            .field_shape(WorthQueryEvidenceTag::new("family"), family.as_str())
            .field_evidence_identity(WorthQueryEvidenceTag::new("source_receipt"), source_receipt)
            .seal()?
            .into(),
    )
}

pub(super) fn causal_observation_query_evidence_identity(
    family: &str,
    source_query: &WorthQueryEvidenceIdentity,
) -> Result<CausalObservationQueryIdentity, IdentityDerivationDenial> {
    Ok(
        WorthQueryEvidenceIdentity::compose(WorthQueryEvidenceScope::CausalObservationQuery)
            .field_shape(WorthQueryEvidenceTag::new("family"), family)
            .field_evidence_identity(WorthQueryEvidenceTag::new("source_query"), source_query)
            .seal()?
            .into(),
    )
}

pub(super) fn causal_observation_basis_evidence_identity(
    posture: &CausalObservationBasisPosture,
    basis_identity: &WorthQueryEvidenceIdentity,
) -> Result<CausalObservationBasisIdentity, IdentityDerivationDenial> {
    Ok(
        WorthQueryEvidenceIdentity::compose(WorthQueryEvidenceScope::CausalObservationBasis)
            .field_shape(WorthQueryEvidenceTag::new("posture"), posture.as_str())
            .field_evidence_identity(WorthQueryEvidenceTag::new("source_basis"), basis_identity)
            .seal()?
            .into(),
    )
}

pub(super) fn causal_evidence_reference_identity_digest(
    family: CausalEvidenceFamily,
    source_reference: &WorthQueryEvidenceIdentity,
) -> Result<CausalEvidenceReferenceDigest, IdentityDerivationDenial> {
    Ok(
        WorthQueryEvidenceIdentity::compose(WorthQueryEvidenceScope::CausalEvidenceReference)
            .field_shape(WorthQueryEvidenceTag::new("family"), family.as_str())
            .field_evidence_identity(
                WorthQueryEvidenceTag::new("source_reference"),
                source_reference,
            )
            .seal()?
            .into(),
    )
}

pub(super) fn write_observation_query_identity(
    inspection: &WorthQueryWriteReceiptInspection,
) -> Result<CausalObservationQueryIdentity, IdentityDerivationDenial> {
    let snapshot_identity = inspection.snapshot_identity().evidence_identity()?;
    let mut encoder =
        WorthQueryEvidenceIdentity::compose(WorthQueryEvidenceScope::CausalObservationQuery)
            .field_shape(WorthQueryEvidenceTag::new("family"), "write_receipt")
            .field_shape(
                WorthQueryEvidenceTag::new("mutation_family"),
                inspection.mutation_family(),
            )
            .field_shape(
                WorthQueryEvidenceTag::new("basis_lane"),
                inspection.basis_lane().as_str(),
            )
            .field_evidence_identity(
                WorthQueryEvidenceTag::new("snapshot_identity"),
                &snapshot_identity,
            );
    if !inspection.mutation_metadata().is_empty() {
        encoder = encoder.field_value_sequence(
            WorthQueryEvidenceTag::new("metadata_entries"),
            inspection
                .mutation_metadata()
                .entries()
                .flat_map(|(key, value)| {
                    [
                        key.as_str().to_string(),
                        value.terminal_digest_text().to_string(),
                    ]
                }),
        );
    }
    Ok(encoder.seal()?.into())
}

pub(super) fn read_observation_receipt_identity(
    receipt: &WorthQueryReadReceipt,
) -> Result<CausalObservationReceiptIdentity, IdentityDerivationDenial> {
    let snapshot_identity = receipt.snapshot_evidence_identity();
    Ok(
        WorthQueryEvidenceIdentity::compose(WorthQueryEvidenceScope::CausalObservationReceipt)
            .field_shape(
                WorthQueryEvidenceTag::new("family"),
                QueryObservationReceiptFamily::ReadReceipt.as_str(),
            )
            .field_evidence_identity(WorthQueryEvidenceTag::new("snapshot"), &snapshot_identity)
            .field_shape(
                WorthQueryEvidenceTag::new("execution_engine"),
                receipt.execution_engine().as_str(),
            )
            .field_value(
                WorthQueryEvidenceTag::new("result"),
                receipt.result_digest(),
            )
            .seal()?
            .into(),
    )
}

pub(super) fn read_observation_query_identity(
    receipt: &WorthQueryReadReceipt,
    snapshot_identity: &WorthQueryEvidenceIdentity,
) -> Result<CausalObservationQueryIdentity, IdentityDerivationDenial> {
    Ok(
        WorthQueryEvidenceIdentity::compose(WorthQueryEvidenceScope::CausalObservationQuery)
            .field_shape(WorthQueryEvidenceTag::new("family"), "read_receipt")
            .field_evidence_identity(WorthQueryEvidenceTag::new("snapshot"), snapshot_identity)
            .field_shape(
                WorthQueryEvidenceTag::new("execution_engine"),
                receipt.execution_engine().as_str(),
            )
            .field_value(WorthQueryEvidenceTag::new("query"), receipt.query_digest())
            .seal()?
            .into(),
    )
}

pub(super) fn read_observation_result_reference_digest(
    receipt: &WorthQueryReadReceipt,
    snapshot_identity: &WorthQueryEvidenceIdentity,
) -> Result<CausalEvidenceReferenceDigest, IdentityDerivationDenial> {
    Ok(
        WorthQueryEvidenceIdentity::compose(WorthQueryEvidenceScope::CausalEvidenceReference)
            .field_shape(
                WorthQueryEvidenceTag::new("family"),
                CausalEvidenceFamily::QueryInspection.as_str(),
            )
            .field_evidence_identity(WorthQueryEvidenceTag::new("snapshot"), snapshot_identity)
            .field_shape(
                WorthQueryEvidenceTag::new("execution_engine"),
                receipt.execution_engine().as_str(),
            )
            .field_value(
                WorthQueryEvidenceTag::new("result"),
                receipt.result_digest(),
            )
            .seal()?
            .into(),
    )
}
