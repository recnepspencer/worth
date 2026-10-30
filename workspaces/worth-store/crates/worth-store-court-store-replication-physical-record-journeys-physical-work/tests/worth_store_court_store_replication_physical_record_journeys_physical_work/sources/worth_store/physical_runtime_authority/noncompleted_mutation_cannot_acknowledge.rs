use worth_store::physical_runtime::{
    IndeterminatePhysicalMutation, ProvenNoEffectPhysicalMutation,
};

fn proven_no_effect_cannot_acknowledge(fate: ProvenNoEffectPhysicalMutation) {
    let _ = fate.into_acknowledgment();
}

fn indeterminate_cannot_acknowledge(fate: IndeterminatePhysicalMutation) {
    let _ = fate.into_acknowledgment();
}

fn main() {}
