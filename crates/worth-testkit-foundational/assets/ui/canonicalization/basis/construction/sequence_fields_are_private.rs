use worth_foundational::{
    CanonicalBasisDomain, CanonicalBasisSequence, CanonicalizationCost,
    CanonicalizationRuleVersion,
};

fn main() {
    let _sequence = CanonicalBasisSequence {
        version: CanonicalizationRuleVersion::new("m2.phase1").unwrap(),
        domain: CanonicalBasisDomain::Value,
        entries: Vec::new(),
        cost: CanonicalizationCost::default(),
    };
}
