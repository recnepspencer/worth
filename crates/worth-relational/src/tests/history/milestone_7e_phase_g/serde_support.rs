use crate::facade::merge::{
    IdentityBasisKind, IdentityBasisScope, IdentityMatchCandidate, IdentityMatchClass,
    IdentityResolutionReason, MergeRecordIdentity, RelationalMergeCorrespondenceWitness,
    RelationalMergeCorrespondenceWitnessPosture, RelationalMergeCorrespondenceWitnessRow,
};
use sha2::{Digest, Sha256};
use worth_relational_contracts::internal_port::merge::data::artifacts::correspondence_witness::RelationalMergeCorrespondenceWitnessOwnerPort as _;

#[derive(Clone, serde::Serialize)]
pub(super) struct CorrespondenceWitnessSerdePayload {
    request_digest: String,
    branch_basis_digest: String,
    rows: Vec<CorrespondenceWitnessRowSerdePayload>,
    witness_digest: String,
}

pub(super) fn witness_payload(
    witness: &RelationalMergeCorrespondenceWitness,
    rows: Vec<CorrespondenceWitnessRowSerdePayload>,
    witness_digest: Option<&str>,
) -> CorrespondenceWitnessSerdePayload {
    let request_digest = witness.request_digest().to_string();
    let branch_basis_digest = witness.branch_basis_digest().to_string();
    let witness_digest = witness_digest.map(ToOwned::to_owned).unwrap_or_else(|| {
        witness_digest_from_row_payloads(&request_digest, &branch_basis_digest, &rows)
    });
    CorrespondenceWitnessSerdePayload {
        request_digest,
        branch_basis_digest,
        rows,
        witness_digest,
    }
}

pub(super) fn recomputed_witness(
    witness: &RelationalMergeCorrespondenceWitness,
    rows: Vec<RelationalMergeCorrespondenceWitnessRow>,
) -> RelationalMergeCorrespondenceWitness {
    crate::merge::data::RelationalMergeCorrespondenceWitness::retained(
        witness.request_digest().to_string(),
        witness.branch_basis_digest().to_string(),
        rows.into(),
    )
}

pub(super) fn forged_row_with_basis(
    row: &RelationalMergeCorrespondenceWitnessRow,
    authority_basis: IdentityBasisKind,
) -> RelationalMergeCorrespondenceWitnessRow {
    let payload = CorrespondenceWitnessRowSerdePayload {
        scope: row.scope().cloned(),
        source_record: row.source_record().clone(),
        target_record: row.target_record().cloned(),
        source: row.source().clone(),
        target: row.target().cloned(),
        match_class: row.match_class().clone(),
        reason: row.reason().clone(),
        authority_basis: authority_basis.clone(),
        candidate_digest: row.candidate_digest().to_string(),
        posture: row.posture(),
    };
    let encoded = rmp_serde::to_vec_named(&payload).expect("encode shifted-basis row");
    let shifted: Result<RelationalMergeCorrespondenceWitnessRow, _> =
        rmp_serde::from_slice(&encoded);
    assert!(shifted.is_err());

    crate::merge::data::row_for_candidate(
        &IdentityMatchCandidate {
            scope: row.scope().cloned(),
            source_record: row.source_record().clone(),
            target_record: row.target_record().cloned(),
            source: row.source().clone(),
            target: row.target().cloned(),
            match_class: row.match_class().clone(),
            reason: row.reason().clone(),
            basis: authority_basis,
        },
        row.posture(),
    )
}

#[derive(Clone, serde::Serialize)]
pub(super) struct CorrespondenceWitnessRowSerdePayload {
    scope: Option<IdentityBasisScope>,
    source_record: crate::transactions::data::RecordRef,
    target_record: Option<crate::transactions::data::RecordRef>,
    source: MergeRecordIdentity,
    target: Option<MergeRecordIdentity>,
    match_class: IdentityMatchClass,
    reason: IdentityResolutionReason,
    authority_basis: IdentityBasisKind,
    candidate_digest: String,
    posture: RelationalMergeCorrespondenceWitnessPosture,
}

pub(super) fn row_payloads(
    rows: &[RelationalMergeCorrespondenceWitnessRow],
    first_row_candidate_digest: Option<&str>,
    first_row_posture: Option<RelationalMergeCorrespondenceWitnessPosture>,
    first_row_basis: Option<IdentityBasisKind>,
) -> Vec<CorrespondenceWitnessRowSerdePayload> {
    rows.iter()
        .enumerate()
        .map(|(index, row)| CorrespondenceWitnessRowSerdePayload {
            scope: row.scope().cloned(),
            source_record: row.source_record().clone(),
            target_record: row.target_record().cloned(),
            source: row.source().clone(),
            target: row.target().cloned(),
            match_class: row.match_class().clone(),
            reason: row.reason().clone(),
            authority_basis: if index == 0 {
                first_row_basis
                    .clone()
                    .unwrap_or_else(|| row.authority_basis().clone())
            } else {
                row.authority_basis().clone()
            },
            candidate_digest: if index == 0 {
                first_row_candidate_digest
                    .unwrap_or(row.candidate_digest())
                    .to_string()
            } else {
                row.candidate_digest().to_string()
            },
            posture: if index == 0 {
                first_row_posture.unwrap_or(row.posture())
            } else {
                row.posture()
            },
        })
        .collect()
}

fn witness_digest_from_row_payloads(
    request_digest: &str,
    branch_basis_digest: &str,
    rows: &[CorrespondenceWitnessRowSerdePayload],
) -> String {
    let mut bytes = Vec::new();
    bytes.extend_from_slice(b"worth.relational.merge.correspondence_witness.v1");
    bytes.extend_from_slice(request_digest.as_bytes());
    bytes.extend_from_slice(branch_basis_digest.as_bytes());
    bytes.extend_from_slice(rows.len().to_string().as_bytes());
    for row in rows {
        bytes.extend_from_slice(
            &rmp_serde::to_vec_named(row)
                .expect("merge correspondence witness row payload must encode"),
        );
    }
    let digest = Sha256::digest(bytes);
    digest.iter().map(|byte| format!("{byte:02x}")).collect()
}
