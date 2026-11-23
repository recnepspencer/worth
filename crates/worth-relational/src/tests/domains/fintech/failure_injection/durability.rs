use super::super::fixture::FintechWorld;

/// Damage the checkpoint the world's durable medium holds, as storage that
/// returns bytes Relational did not write. Reports whether one was filed.
pub(crate) fn damage_filed_checkpoint(world: &FintechWorld) -> bool {
    let Some(medium) = world.medium.as_ref() else {
        return false;
    };
    let filed = medium
        .try_recovery_source()
        .expect("intact artifact frames")
        .checkpoint()
        .is_some();
    if filed {
        medium.damage_checkpoint_payload();
    }
    filed
}
