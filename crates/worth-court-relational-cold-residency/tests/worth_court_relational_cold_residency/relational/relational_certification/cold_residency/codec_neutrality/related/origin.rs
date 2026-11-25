//! Descriptive origin read from the actual canonical commit selected by its holder.
use serde::{Deserialize, Serialize};
use worth_execution::ExecutionRequest;
use worth_relational::facade::{
    durability::RelationalCanonicalCommitArtifact,
    history::{CanonicalCommitEnvelope, CommitId},
    identity::{EntityId, KindId, RelationId, VersionId},
    mvcc::{RelationalRequiredOutputPublication, RequiredOutputPublicationMeaning},
    runtime::RelationalRuntime,
};

#[derive(Deserialize)]
pub(super) struct Origin {
    pub(super) birth: CommitId,
    pub(super) commit: worth_relational::facade::history::RelationalCommitReceipt,
    pub(super) outputs: Vec<EntityId>,
    pub(super) relations: Vec<RelationOrigin>,
    pub(super) meaning: RequiredOutputPublicationMeaning,
}

#[derive(Clone, Deserialize, Serialize)]
pub(super) struct RelationOrigin {
    pub(super) id: RelationId,
    pub(super) kind: KindId,
    pub(super) source: EntityId,
    pub(super) target: EntityId,
    pub(super) birth: CommitId,
    pub(super) before_write: Option<VersionId>,
    pub(super) after_write: VersionId,
}

#[derive(Deserialize)]
struct EnvelopeProjection {
    required_output_origin: Option<Origin>,
}

pub(super) fn single_origin(envelope: &CanonicalCommitEnvelope) -> Option<Origin> {
    let bytes = rmp_serde::to_vec_named(envelope).unwrap();
    rmp_serde::from_slice::<EnvelopeProjection>(&bytes)
        .unwrap()
        .required_output_origin
}

pub(super) fn exact_origin(
    execution: &ExecutionRequest<'_, '_>,
    runtime: &RelationalRuntime,
    publication: &RelationalRequiredOutputPublication,
) -> Origin {
    let (commit, ordinal) = publication.authority().publication_coordinate();
    assert_eq!(ordinal, 0);
    let artifact = runtime
        .durability()
        .canonical_commit_artifacts(execution)
        .map(Result::unwrap)
        .find(|artifact| artifact.commit_id() == CommitId(commit))
        .unwrap();
    let readmitted =
        RelationalCanonicalCommitArtifact::readmit(artifact.as_bytes(), execution).unwrap();
    let origin = single_origin(readmitted.envelope()).unwrap();
    assert_eq!(&origin.meaning, publication.meaning());
    origin
}

pub(super) fn coupled_origins(envelope: &CanonicalCommitEnvelope) -> Vec<Origin> {
    #[derive(Deserialize)]
    struct CoupledEnvelope {
        coupled_authored_publication: Coupled,
    }
    #[derive(Deserialize)]
    struct Coupled {
        participants: Vec<Participant>,
    }
    #[derive(Deserialize)]
    struct Participant {
        origin: Origin,
    }
    let bytes = rmp_serde::to_vec_named(envelope).unwrap();
    rmp_serde::from_slice::<CoupledEnvelope>(&bytes)
        .unwrap()
        .coupled_authored_publication
        .participants
        .into_iter()
        .map(|participant| participant.origin)
        .collect()
}
