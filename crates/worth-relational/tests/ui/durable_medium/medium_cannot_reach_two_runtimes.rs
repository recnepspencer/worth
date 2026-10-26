#[path = "../durable_medium_journal.rs"]
mod journal;
use journal::Journal;
use worth_relational::facade::durability::RelationalDurableMedium;
use worth_relational::facade::runtime::RelationalRuntimeApi;

fn main() {
    let medium: Box<dyn RelationalDurableMedium> = Box::new(Journal);
    let builder = RelationalRuntimeApi::builder();
    let _first = builder.clone().build_durable(medium);
    let _second = builder.build_durable(medium);
}
