mod physical_ack_mapping;
mod redo_fixture;
pub(in crate::courtroom::protocol_models) mod scenario;

use physical_ack_mapping::map_physical_mutation_acknowledgment;

#[cfg(test)]
mod tests;
