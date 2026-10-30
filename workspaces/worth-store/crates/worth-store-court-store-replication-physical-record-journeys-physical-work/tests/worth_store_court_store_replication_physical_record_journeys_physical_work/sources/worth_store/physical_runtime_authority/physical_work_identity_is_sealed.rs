use std::num::NonZeroU64;

use worth_store::physical_runtime::{
    PhysicalOperationIdentity, PhysicalSignalProfileIdentity, PhysicalWorkGeneration,
};

fn main() {
    let sequence = NonZeroU64::new(1).unwrap();
    let _operation = PhysicalOperationIdentity::from_owner_sequence(sequence);
    let _profile = PhysicalSignalProfileIdentity([0; 32]);
    let _generation = PhysicalWorkGeneration::from_lifecycle(todo!());
}
