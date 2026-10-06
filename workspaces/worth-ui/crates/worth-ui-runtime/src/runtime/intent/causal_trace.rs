use worth_ui_inspection::{
    UiIntentCausalTraceOperabilityEvidence, UiIntentCausalTraceOperabilityPosture,
    UiIntentCausalTracePayloadEvidence, UiIntentCausalTraceRouteEvidence,
    UiIntentEvidenceReference,
};

pub(crate) struct UiIntentCausalTraceAdmissionPrefix {
    pub(crate) reference: UiIntentEvidenceReference,
    pub(crate) route: UiIntentCausalTraceRouteEvidence,
    pub(crate) payload: UiIntentCausalTracePayloadEvidence,
    pub(crate) operability: UiIntentCausalTraceOperabilityEvidence,
}

impl UiIntentCausalTraceAdmissionPrefix {
    pub(crate) fn from_candidate(
        candidate: &super::UiCurrentIntentAdmissionCandidate,
    ) -> Option<Self> {
        let basis = candidate.prepared().payload().input_basis();
        let reference = basis.evidence_reference()?;
        let owner_revisions = basis.owner_revisions();
        Some(Self {
            reference,
            route: UiIntentCausalTraceRouteEvidence::new(
                candidate.graph_node().digest(),
                digest_text(candidate.definition_id().as_str()),
                digest_text(candidate.declaration_identity_value().as_str()),
            ),
            payload: UiIntentCausalTracePayloadEvidence::new(
                u8::try_from(owner_revisions.len()).expect("intent payload field limit fits u8"),
                owner_revisions.first().map(primary_revision),
                owner_revision_digest(owner_revisions),
                basis.cost().admitted_utf8_bytes(),
            ),
            operability: UiIntentCausalTraceOperabilityEvidence::new(
                UiIntentCausalTraceOperabilityPosture::Operable,
                candidate.decision().cost().selected_dependencies_visited(),
                decision_digest(candidate.decision()),
            ),
        })
    }
}

fn primary_revision(revision: &super::UiIntentInputOwnerRevision) -> u64 {
    match revision {
        super::UiIntentInputOwnerRevision::Query(revision) => {
            revision.revision().observation_order()
        }
        super::UiIntentInputOwnerRevision::Application(revision) => revision.revision(),
        super::UiIntentInputOwnerRevision::Draft(revision) => revision.draft_revision(),
        super::UiIntentInputOwnerRevision::Expression(revision) => revision.outcome_revision(),
    }
}

fn owner_revision_digest(revisions: &[super::UiIntentInputOwnerRevision]) -> u64 {
    revisions.iter().fold(FNV_OFFSET, |digest, revision| {
        let (tag, field, owner, revision) = match revision {
            super::UiIntentInputOwnerRevision::Query(revision) => (
                1,
                revision.field().stable_name(),
                revision.revision().projection_identity().as_str(),
                revision.revision().observation_order(),
            ),
            super::UiIntentInputOwnerRevision::Application(revision) => (
                2,
                revision.field().stable_name(),
                revision.identity(),
                revision.revision(),
            ),
            super::UiIntentInputOwnerRevision::Draft(revision) => (
                3,
                revision.field().stable_name(),
                "committed-draft",
                revision.draft_revision(),
            ),
            super::UiIntentInputOwnerRevision::Expression(revision) => (
                4,
                revision.field().stable_name(),
                revision.identity(),
                revision.outcome_revision(),
            ),
        };
        fold_u64(
            fold_text(fold_text(fold_u64(digest, tag), field), owner),
            revision,
        )
    })
}

fn decision_digest(decision: &super::UiIntentOperabilityDecision) -> u64 {
    fold_u64(
        fold_text(FNV_OFFSET, decision.contract_identity()),
        u64::try_from(decision.cost().selected_dependencies_visited()).unwrap_or(u64::MAX),
    )
}

const FNV_OFFSET: u64 = 0xcbf29ce484222325;
const FNV_PRIME: u64 = 0x100000001b3;

fn digest_text(value: &str) -> u64 {
    fold_text(FNV_OFFSET, value)
}

fn fold_text(mut digest: u64, value: &str) -> u64 {
    for byte in value.bytes() {
        digest ^= u64::from(byte);
        digest = digest.wrapping_mul(FNV_PRIME);
    }
    digest
}

fn fold_u64(digest: u64, value: u64) -> u64 {
    value
        .to_le_bytes()
        .into_iter()
        .fold(digest, |mut digest, byte| {
            digest ^= u64::from(byte);
            digest.wrapping_mul(FNV_PRIME)
        })
}
