#[path = "../durable_medium_journal.rs"]
mod journal;
use journal::Journal;
use worth_relational::facade::durability::RelationalDurableMedium;
use worth_relational::facade::runtime::RelationalRuntimeApi;

fn main() {
    let medium: Box<dyn RelationalDurableMedium> = Box::new(Journal);
    let _first = RelationalRuntimeApi::builder().build_durable(medium.clone());
    let _second = RelationalRuntimeApi::builder().build_durable(medium);
}
