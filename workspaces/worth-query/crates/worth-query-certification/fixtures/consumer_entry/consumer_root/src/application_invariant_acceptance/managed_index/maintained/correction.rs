//! Recorded correction extends the existing maintained Catalog and owning Program.
use super::{host, query, schema::*};
use worth_query_decl::facade::{
    application_aftermath::*, application_program::*, application_schema::*,
};
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
        .correction::<recorded::Note>()
        .correction::<recorded::Mover>()
        .correction::<recorded::Motion>()
}
pub(super) fn visit<V: ApplicationCorrectionVisitor<Catalog>>(
    source: std::any::TypeId,
    visitor: V,
) -> Result<V::Output, V> {
    recorded::visit(source, visitor)
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
pub type NoteAction = ApplicationActionReference<
    Catalog,
    ApplicationRootComposition,
    host::Feature,
    note::Binding,
    NoteDelivery,
>;
pub type MoverAction = ApplicationActionReference<
    Catalog,
    ApplicationRootComposition,
    host::Feature,
    mover::Binding,
    MoverDelivery,
>;

pub struct MotionDelivery;
impl ApplicationActionDelivery<Catalog, motion::Binding> for MotionDelivery {
    type Mode = ApplicationSourceCompletion;
    const IDENTITY: &'static str = "maintained-catalog.recorded-motion.delivery.v1";
}
pub type MotionAction = ApplicationActionReference<
    Catalog,
    ApplicationRootComposition,
    host::Feature,
    motion::Binding,
    MotionDelivery,
>;
