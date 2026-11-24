mod contexts;
mod decisions;
mod resolution;
mod selected_basis;
#[cfg(test)]
mod tests;
mod value_strategy;

pub(crate) use decisions::current_topology_rewire_admission_policy;

fn binding_matches_aspect(
    binding: &crate::schema::data::LoweredAspectContractBinding,
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
