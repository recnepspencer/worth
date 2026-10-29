#[expect(
    dead_code,
    reason = "this target uses genesis fixtures; other recovery targets exercise checkpoint, WAL and topology fixtures"
)]
#[allow(dead_code, unused_imports)]
#[path = "synthetic_topology/mod.rs"]
pub(crate) mod synthetic_topology;
