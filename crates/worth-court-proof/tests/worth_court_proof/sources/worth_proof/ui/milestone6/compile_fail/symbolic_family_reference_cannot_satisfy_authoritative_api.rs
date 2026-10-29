use worth_proof::{AuthoritativeFamilyMember, CompositionFamilySymbol};

fn requires_authoritative_member(member: AuthoritativeFamilyMember<u64>) {
    let _ = member;
}

fn main() {
    let symbol = CompositionFamilySymbol::new(7_u64);
    requires_authoritative_member(symbol);
}
