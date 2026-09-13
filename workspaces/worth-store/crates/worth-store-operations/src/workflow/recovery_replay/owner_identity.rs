use sha2::{Digest, Sha256};

use crate::workflow::{
    ExactRecoveryFrontier, StagedWalApplicationReceipt, StagedWalReplaySourceReceipt,
};

pub(crate) fn replay_owner_identity(
    domain: &[u8],
    plan_fingerprint: [u8; 32],
    frontier: ExactRecoveryFrontier,
    replay: StagedWalReplaySourceReceipt,
    application: StagedWalApplicationReceipt,
) -> [u8; 32] {
    fingerprint(domain, |digest| {
        digest.update(plan_fingerprint);
        digest.update(frontier.identity());
        digest.update(frontier.checkpoint_durability().to_be_bytes());
        digest.update(frontier.wal_structural().to_be_bytes());
        digest.update(frontier.local_durable_commit().to_be_bytes());
        digest.update(frontier.client_acknowledged().to_be_bytes());
        digest.update(frontier.replication_acknowledged().to_be_bytes());
        digest.update(frontier.authority_identity().fingerprint());
        digest.update(frontier.source_lineage());
        digest.update(replay.identity());
        digest.update(replay.manifest_digest());
        digest.update(replay.frame_count().to_be_bytes());
        digest.update(replay.bytes_verified().to_be_bytes());
        digest.update(replay.interval().0.to_be_bytes());
        digest.update(replay.interval().1.to_be_bytes());
        digest.update(application.identity());
        digest.update(application.application_identity());
        digest.update(application.replay_source_identity());
        digest.update(application.resulting_frontier_identity());
        digest.update(application.applied_frames().to_be_bytes());
    })
}

pub(crate) fn fingerprint(domain: &[u8], update: impl FnOnce(&mut Sha256)) -> [u8; 32] {
    let mut digest = Sha256::new();
    digest.update(domain);
    update(&mut digest);
    digest.finalize().into()
}
