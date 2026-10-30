use worth_foundational::{
    AbsenceLaw, AspectContract, AspectContractRevision, AspectEquivalenceBasis,
    AspectEvolutionPolicy, AspectIdentity, AspectKey, AspectMaskContract, AspectShape,
    ScalarAspectType,
};

fn main() {
    let key = AspectKey::new("count").unwrap();

    let _contract = AspectContract::new(
        key,
        AspectIdentity(1),
        AspectContractRevision(1),
        AspectShape::Scalar(ScalarAspectType::Int64),
        AspectMaskContract::opaque_diagnostic_only(),
        AbsenceLaw::Required,
        AspectEquivalenceBasis::OpaqueIdentity,
        AspectEvolutionPolicy::AdditiveFieldsAllowed,
    );
}
