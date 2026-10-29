use worth_store_authority::CanonicalAuthorityRecord;
use worth_store_contracts::StableArtifactId;

fn main() {
    let artifact_id = StableArtifactId::new("sha256:terminal-projection-digest").unwrap();

    let _record = CanonicalAuthorityRecord::from_current_authority(artifact_id.clone(), artifact_id);
}
