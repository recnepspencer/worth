#[path = "../durable_medium_journal.rs"]
mod journal;
use journal::Journal;
use worth_relational::facade::runtime::RelationalRuntimeApi;

fn main() {
    let builder = RelationalRuntimeApi::builder();
    // A builder still clones: it never holds a medium.
    let _memory_only = builder.clone().build();
    let _durable = builder.build_durable(Box::new(Journal));
}
