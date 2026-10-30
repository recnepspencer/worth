use worth_store_authority::CanonicalAuthorityRecord;
use worth_store_contracts::StableArtifactId;

fn main() {
    let artifact_id = StableArtifactId::new("sha256:terminal-projection-digest").unwrap();

    let _record = CanonicalAuthorityRecord::new(artifact_id);
}
