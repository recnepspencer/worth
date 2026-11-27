mod allocation_attribution;
pub(crate) mod checkpoint_stage;
mod composed_world;
mod durability_append_matrix;
mod embedded_cost;
mod index_retirement;
mod naming_scaling;
#[path = "relational_certification/mod.rs"]
pub(crate) mod relational_certification;
mod restoration_header;
mod source_initial_inputs;

mod schema_projection;

mod incremental_indexes;
mod kind_directory;
mod kind_maintenance;

mod pinned_projection;

mod cold_checkpoint;
mod lazy_pin;
mod reopen_growth;
mod snapshot_counts;

mod symbol_admission;
mod symbol_scaling;

mod shared_aspect_reads;

mod global_laws;
