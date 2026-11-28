//! Note and movement actions for the maintained Catalog and owning Program.
use super::{query, schema::*};
use worth_query_decl::facade::{application_program::*, application_schema::*};
use worth_query_host::facade::{application_contribution::*, primary_graph::*};
pub(super) mod motion;
pub(super) mod mover;
pub(super) mod note;
pub(super) mod recorded;
pub(super) mod state;
pub(super) fn declare(
    schema: ApplicationSchemaDeclarationBuilder<Catalog>,
) -> ApplicationSchemaDeclarationBuilder<Catalog> {
    state::declare(schema)
}

pub(super) fn configure(
    setup: &mut WorthQueryApplicationContributionSetup<'_, Catalog>,
) -> Result<(), WorthQueryPrimaryGraphInstallationDenial> {
    setup.handler::<note::Binding, _>(note::Handler)?;
    setup.handler::<mover::Binding, _>(mover::Handler)?;
    setup.handler::<motion::Binding, _>(motion::Handler)
}
pub struct NoteDelivery;
impl ApplicationActionDelivery<Catalog, note::Binding> for NoteDelivery {
    type Mode = ApplicationSourceCompletion;
    const IDENTITY: &'static str = "maintained-catalog.recorded-note.delivery.v1";
}
pub struct MoverDelivery;
impl ApplicationActionDelivery<Catalog, mover::Binding> for MoverDelivery {
    type Mode = ApplicationSourceCompletion;
    const IDENTITY: &'static str = "maintained-catalog.recorded-mover.delivery.v1";
}
pub struct MotionDelivery;
impl ApplicationActionDelivery<Catalog, motion::Binding> for MotionDelivery {
    type Mode = ApplicationSourceCompletion;
    const IDENTITY: &'static str = "maintained-catalog.recorded-motion.delivery.v1";
}
