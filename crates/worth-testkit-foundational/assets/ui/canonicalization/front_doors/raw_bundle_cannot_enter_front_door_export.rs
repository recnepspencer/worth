use worth_foundational::{
    canonicalization, CanonicalBasisBundle, CanonicalEquivalenceBasis, CanonicalProducerShape,
};

fn impossible<T>() -> T {
    panic!("type-check only")
}

fn main() {
    let bundle: CanonicalBasisBundle = impossible();

    let _ = canonicalization()
        .export()
        .from_bundle(bundle)
        .named("fixture")
        .for_producer_shape(CanonicalProducerShape::GoldenFixture)
        .under(CanonicalEquivalenceBasis::ExactCanonicalBasis);
}
