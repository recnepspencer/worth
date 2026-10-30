use worth_foundational::{
    aspects, AbsenceLaw, AspectEquivalenceBasis, AspectEvolutionPolicy, ScalarAspectType,
};

fn main() {
    let vocabulary = aspects().vocabulary();
    let _contract = aspects()
        .contract()
        .for_key(vocabulary.key("count").unwrap())
        .identified_by(vocabulary.identity(1))
        .at_revision(vocabulary.revision(1))
        .custom(
            worth_foundational::AspectShape::Scalar(ScalarAspectType::Int64),
            aspects().mask_contract().scalar(),
            AbsenceLaw::Required,
            AspectEquivalenceBasis::ExactCanonicalValue,
            AspectEvolutionPolicy::ExplicitBreakRequired,
        );
}
