use sha2::{Digest, Sha256};
use worth_foundational::{
    prepare_canonical_basis_sequence, CanonicalBasisConstructionDenial, CanonicalBasisDomain,
    CanonicalBasisEntry, CanonicalBasisEntryKind, CanonicalBasisLocus, CanonicalBasisValue,
    CanonicalDigestId, CanonicalIntegerWidth, CanonicalizationRuleVersion,
};
use worth_proof::TransitionOutcome;

use crate::merge::data::{
    RelationalMergeAdmittedSurfaceRow, RelationalMergeProofPacket,
    RelationalMergeProofPacketAdmissionPosture, RelationalMergeProofPacketCanonicalBasis,
};

use super::MergeAccess;

const PHASE_6_CANONICAL_RULE_VERSION: &str = "worth.relational.merge.proof_packet.canonical.v1";

impl<'runtime> MergeAccess<'runtime> {
    pub fn lower_merge_proof_packet_to_foundational_canonical_basis(
        &self,
        packet: &RelationalMergeProofPacket,
    ) -> TransitionOutcome<RelationalMergeProofPacketCanonicalBasis, CanonicalBasisConstructionDenial>
    {
        let version = CanonicalizationRuleVersion::new(PHASE_6_CANONICAL_RULE_VERSION)
            .expect(/* Proof: This constructor receives a fixed nonempty literal in the required grammar and version/domain; it has no runtime-dependent input. */ "phase 6 canonical rule version must be valid");
        prepare_canonical_basis_sequence(
            version,
            CanonicalBasisDomain::Transition,
            merge_proof_packet_canonical_entries(packet),
        )
        .map_success(RelationalMergeProofPacketCanonicalBasis::new)
    }
}

fn merge_proof_packet_canonical_entries(
    packet: &RelationalMergeProofPacket,
) -> Vec<CanonicalBasisEntry> {
    let branch_basis_digest = packet.branch_basis().basis_digest();
    let mut entries = vec![
        digest_entry("merge.packet.digest", packet.packet_digest()),
        digest_entry("merge.request.digest", packet.request().request_digest()),
        digest_entry("merge.branch_basis.digest", &branch_basis_digest),
        digest_entry(
            "merge.correspondence_witness.digest",
            packet.correspondence_witness_digest(),
        ),
        digest_entry(
            "merge.schema_reconciliation_witness.digest",
            packet.schema_reconciliation_witness_digest(),
        ),
        digest_entry(
            "merge.strategy_witness.digest",
            packet.strategy_witness_digest(),
        ),
        text_entry(
            "merge.request_lowering.digest",
            packet.foundational_request_lowering_digest(),
        ),
        digest_entry(
            "merge.admitted_surface.digest",
            packet.admitted_merge_surface_digest(),
        ),
        u64_entry(
            "merge.admitted_surface.count",
            packet.admitted_merge_surface().len() as u64,
        ),
        digest_entry("merge.planning.digest", packet.planning_digest()),
        digest_entry("merge.execution.digest", packet.execution_digest()),
        text_entry(
            "merge.admission_posture",
            canonical_admission_posture(packet.admission_posture()),
        ),
    ];

    for (index, row) in packet.admitted_merge_surface().iter().enumerate() {
        entries.push(digest_entry(
            &format!("merge.admitted_surface.row.{index}.digest"),
            &admitted_surface_row_digest(row),
        ));
    }

    entries
}

fn admitted_surface_row_digest(row: &RelationalMergeAdmittedSurfaceRow) -> String {
    let mut bytes = Vec::new();
    bytes.extend_from_slice(b"worth.relational.merge.admitted_surface.row.v1");
    bytes.extend_from_slice(
        &rmp_serde::to_vec_named(row).expect(/* Proof: The serialized value contains only closed native scalar, sequence, map and derived-Serialize record fields; encoding uses an in-memory Vec/hash writer, with no user serializer or fallible I/O. */ "merge admitted surface row must encode"),
    );
    let digest = Sha256::digest(bytes);
    digest.iter().map(|byte| format!("{byte:02x}")).collect()
}

fn canonical_admission_posture(
    posture: RelationalMergeProofPacketAdmissionPosture,
) -> &'static str {
    match posture {
        RelationalMergeProofPacketAdmissionPosture::ExecutionAdmitted => "execution_admitted",
    }
}

fn text_entry(locus: &str, value: &str) -> CanonicalBasisEntry {
    CanonicalBasisEntry::new(
        CanonicalBasisDomain::Transition,
        CanonicalBasisLocus::Named(locus.to_string().into()),
        CanonicalBasisEntryKind::TransitionArtifact,
        CanonicalBasisValue::ExactText(value.to_string().into()),
    )
}

fn u64_entry(locus: &str, value: u64) -> CanonicalBasisEntry {
    CanonicalBasisEntry::new(
        CanonicalBasisDomain::Transition,
        CanonicalBasisLocus::Named(locus.to_string().into()),
        CanonicalBasisEntryKind::TransitionArtifact,
        CanonicalBasisValue::UnsignedInteger {
            width: CanonicalIntegerWidth::Bits64,
            value: u128::from(value),
        },
    )
}

fn digest_entry(locus: &str, value: &str) -> CanonicalBasisEntry {
    CanonicalBasisEntry::new(
        CanonicalBasisDomain::Transition,
        CanonicalBasisLocus::Named(locus.to_string().into()),
        CanonicalBasisEntryKind::TransitionArtifact,
        CanonicalBasisValue::BytesDigest(canonical_digest_id(value)),
    )
}

fn canonical_digest_id(value: &str) -> CanonicalDigestId {
    assert_eq!(
        /* Proof: RelationalMergeProofPacket fields are private. Its builder computes SHA-256 lowercase hex; its custom Deserialize validates the supplied hex digests before construction and computes the remaining digests with the same encoder. This function never accepts an arbitrary facade string. */
        value.len(),
        64,
        "canonical merge proof packet digests must be lowercase sha256 hex",
    );
    let mut bytes = [0_u8; 32];
    for (index, slot) in bytes.iter_mut().enumerate() {
        let start = index * 2;
        let end = start + 2;
        *slot = u8::from_str_radix(&value[start..end], 16)
            .expect(/* Proof: This string is a SHA-256 digest generated by the private canonical encoder as exactly 64 lowercase hexadecimal ASCII characters. */ "canonical merge proof packet digests must be valid lowercase sha256 hex");
    }
    CanonicalDigestId::new(bytes)
}
