//! Worlds killed while Serving moved selected media: an inline rewrite and a
//! final extent copy, each killed once its WAL was durable and before its
//! root published. Completing either reads the moved bytes again.

#[allow(unused_imports)] use worth_store_testkit_recovery_runtime::production_entry::published_above_checkpoint::limit_sweep::moved_media::*;

#[test]
fn moved_media_child() {
    let Some(marker) = std::env::var_os(MARKER_ENV) else {
        return;
    };
    let marker = std::path::PathBuf::from(marker);
    match std::env::var(ROLE_ENV).as_deref() {
        Ok("rewrite") => park_rewrite(&marker),
        Ok("copy") => park_copy(&marker),
        role => panic!("unknown moved-media role {role:?}"),
    }
}
