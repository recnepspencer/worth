pub mod contexts;
pub mod decisions;
pub mod resolution;

#[cfg(test)]
pub mod tests;
pub mod value_strategy;

pub use decisions::current_topology_rewire_admission_policy;

fn binding_matches_aspect(
    binding: &worth_relational_contracts::internal_port::schema::data::aspect_contracts::lowered_plan::LoweredAspectContractBinding,
    aspect_key: &worth_foundational::facade::AspectKey,
) -> bool {
    aspect_key_equivalent(binding.aspect_key(), aspect_key)
}

fn aspect_key_equivalent(
    left: &worth_foundational::facade::AspectKey,
    right: &worth_foundational::facade::AspectKey,
) -> bool {
    left == right
}
