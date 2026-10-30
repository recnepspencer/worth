use worth_store_physical_backend::PhysicalDurabilityAdmissionBasis;

fn duplicate(basis: PhysicalDurabilityAdmissionBasis) {
    let duplicate = basis;
    let _ = (basis, duplicate);
}

fn main() {}
