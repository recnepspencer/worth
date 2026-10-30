use worth_store_authority::StoreCurrentAuthorityWitness;

fn require_current_authority(_: StoreCurrentAuthorityWitness) {}

fn main() {
    let digest_text = String::from("sha256:projection-digest");
    require_current_authority(digest_text);
}
