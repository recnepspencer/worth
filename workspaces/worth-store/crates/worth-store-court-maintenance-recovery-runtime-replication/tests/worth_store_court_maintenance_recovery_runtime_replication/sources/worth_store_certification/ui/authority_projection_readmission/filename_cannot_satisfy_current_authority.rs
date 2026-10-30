use worth_store_authority::{StoreAuthorityFilename, StoreCurrentAuthorityWitness};

fn require_current_authority(_: StoreCurrentAuthorityWitness) {}

fn main() {
    let filename = StoreAuthorityFilename::imported_filename("store-authority.snapshot");
    require_current_authority(filename);
}
