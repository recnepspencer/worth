use worth_store_authority::{StoreCurrentAuthorityWitness, StoreExternalAuthorityToken};

fn require_current_authority(_: StoreCurrentAuthorityWitness) {}

fn main() {
    let external_token = StoreExternalAuthorityToken::imported("store.phase8.external.identity");
    require_current_authority(external_token);
}
