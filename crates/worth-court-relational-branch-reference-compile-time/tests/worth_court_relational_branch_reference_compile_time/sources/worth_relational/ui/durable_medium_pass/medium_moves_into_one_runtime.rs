#[path = "../durable_medium_journal.rs"]
mod journal;
use journal::Journal;
use worth_relational::facade::runtime::RelationalRuntimeApi;

pub(crate) fn main() {
    let builder = RelationalRuntimeApi::builder();
    // A builder still clones: it never holds a medium.
    let _memory_only = builder
        .clone()
        .build()
        .expect("valid native runtime configuration");
    let _durable = builder
        .build_durable(Box::new(Journal))
        .expect("valid native runtime configuration");
}
