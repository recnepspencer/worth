use crate::facade::runtime::RelationalRuntime;

/// Damage the checkpoint the world's durable medium holds, as storage that
/// returns bytes Relational did not write. Reports whether one was filed.
pub(crate) fn damage_filed_checkpoint(runtime: &RelationalRuntime) -> bool {
    let medium = crate::tests::support::attached_test_medium(runtime);
    let filed = medium
        .recovery_source()
        .expect("intact artifact frames")
        .checkpoint()
        .is_some();
    if filed {
        medium.damage_checkpoint_payload();
    }
    filed
}
