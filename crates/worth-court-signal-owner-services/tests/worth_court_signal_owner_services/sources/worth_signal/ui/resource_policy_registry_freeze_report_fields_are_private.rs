use worth_signal::facade::core::{
    FrozenResourcePolicyRegistry, ResourcePolicyRegistryFreezeReport,
};

fn main() {
    let registry = FrozenResourcePolicyRegistry::built_in();
    let _report = ResourcePolicyRegistryFreezeReport {
        descriptor_count: 1,
        id_index_width: 1,
        kind_name_index_width: 1,
        registry_digest: registry.registry_digest().clone(),
    };
}
