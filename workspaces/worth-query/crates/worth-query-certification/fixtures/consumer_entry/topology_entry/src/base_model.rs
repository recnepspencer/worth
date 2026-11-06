//! Shared topology meaning used by the consumer and native owner fixture.
#[path = "base_model/rows.rs"]
pub(super) mod rows;
use super::fixture_values::PositiveLength;
pub use super::storage_model::*;
pub use rows::*;
use worth_query_decl::facade::application_schema::*;

/// What the shared topology model asks of the schema that declares it. A
/// program's own schema policy, such as the consumer fixture's
/// `TopologySchemaBinding`, stays with that program.
pub trait TopologyModelBinding: ApplicationSchema {
    /// The work the checked planar invariant may spend.
    fn planar_invariant_work() -> Option<std::num::NonZeroU64> {
        std::num::NonZeroU64::new(super::planar_topology::PLANAR_CHECKED_WORK)
    }
}

use super::TopologyLengthBinding;

pub(super) fn declare<Schema: TopologyModelBinding>(
    schema: ApplicationSchemaDeclarationBuilder<Schema>,
) -> ApplicationSchemaDeclarationBuilder<Schema> {
    use super::*;
    super::planar_islands::declare(schema)
        .invariant(planar_turn_invariant::<Schema>())
        .principal_binding(ConsumerPrincipalBinding::reference::<Schema>())
}
