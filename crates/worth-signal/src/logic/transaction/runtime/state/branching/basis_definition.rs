use super::super::runtime_state::SignalRuntime;
use crate::schema::data::SignalSchemaRegistry;

pub(crate) fn signal_definition_basis<D, I, E, Ctx, T>(
    runtime: &SignalRuntime<D, I, E, Ctx, T>,
) -> u64
where
    D: Copy + Ord + std::fmt::Debug + 'static,
    I: Copy + Ord,
    T: Copy + Ord,
{
    signal_definition_basis_from_registry(&runtime.schema_registry)
}

pub(in crate::logic::transaction::runtime) fn signal_definition_basis_from_registry(
    schema_registry: &SignalSchemaRegistry,
) -> u64 {
    schema_registry
        .registry_digest()
        .as_bytes()
        .iter()
        .take(8)
        .fold(0_u64, |value, byte| {
            value.wrapping_mul(257).wrapping_add(u64::from(*byte))
        })
}
