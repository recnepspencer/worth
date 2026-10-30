use worth_foundational::{BoundaryEpoch, EquivalenceBasisId};

fn needs_epoch(_epoch: BoundaryEpoch) {}

fn main() {
    let basis_id = EquivalenceBasisId::new(1);
    needs_epoch(basis_id);
}
