use worth_foundational::{
    AspectContract, AspectContractRevision, AspectIdentity, AspectKey, AspectMask, MutationMask,
    ScalarAspectType,
};

fn main() {
    let contract = AspectContract::scalar(
        AspectKey::new("title").unwrap(),
        AspectIdentity(1),
        AspectContractRevision(1),
        ScalarAspectType::String,
    );
    let mask = AspectMask::<MutationMask>::whole_aspect();
    let _ = contract.admits_projection_mask(&mask);
}
