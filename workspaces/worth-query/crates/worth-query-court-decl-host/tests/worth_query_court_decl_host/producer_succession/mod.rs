//! Generator succession through the public installed Query producer path.
use std::sync::{Arc, Mutex};
use worth_query_decl::facade::application_aftermath::*;
use worth_query_decl::facade::{
    application_operation::*, application_program::*, application_query::*, application_schema::*,
    *,
};
use worth_query_host::facade::{
    application_contribution::*, application_entry::*, application_installation as installation,
    primary_graph::*,
};
#[macro_use]
mod readiness;
#[macro_use]
mod retirement;
mod checkpoint;
mod checkpoint_value;
mod correction;
mod correction_courts;
mod courts;
mod history_courts;
mod independent_generator_courts;
mod producer;
mod producers;
mod program;
mod query;
mod realization;
mod realization_read;
mod refusal_courts;
mod schema;
mod source_control;
mod world;
use producer::*;
use program::Program;
use query::*;
use retirement::Retirement;
use schema::*;
#[derive(Clone, Debug)]
pub struct Selection {
    pub key: String,
    pub tier: u8,
    pub preserve: bool,
    pub prior: bool,
}
pub type Selections = Arc<Mutex<Vec<Selection>>>;
pub struct Contribution;
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Fault {
    None,
    Missing,
    MissingLeaf,
    Partial,
    Foreign,
    Extra,
    HiddenDeletion,
    Repeated,
}
pub struct Configuration {
    pub selections: Selections,
    pub fault: Fault,
    pub child_width: u64,
}
impl ApplicationSchemaContribution<Schema> for Contribution {
    const IDENTITY: ApplicationSchemaContributionIdentity =
        ApplicationSchemaContributionIdentity::new("succession.contribution");
    fn register_members(
        schema: ApplicationSchemaDeclarationBuilder<Schema>,
    ) -> ApplicationSchemaDeclarationBuilder<Schema> {
        schema::declare(schema)
    }
    fn visit_correction<V: ApplicationCorrectionVisitor<Schema>>(
        source: std::any::TypeId,
        visitor: V,
    ) -> Result<V::Output, V> {
        if source == std::any::TypeId::of::<source_control::Binding>() {
            Ok(dispatch_application_correction::<
                Schema,
                correction::Resize,
                V,
            >(visitor))
        } else {
            Err(visitor)
        }
    }
}
impl WorthQueryApplicationContribution<Schema> for Contribution {
    type Configuration = Configuration;
    fn contracts(
        c: &mut WorthQueryApplicationContributionContracts<Schema>,
    ) -> Result<(), WorthQueryPrimaryGraphInstallationDenial> {
        producers::parent::contracts(c)?;
        producers::child::contracts(c)?;
        producers::leaf::contracts(c).and_then(|()| producers::independent_parent::contracts(c))
    }
    fn configure(
        configuration: Configuration,
        setup: &mut WorthQueryApplicationContributionSetup<'_, Schema>,
    ) -> Result<(), WorthQueryPrimaryGraphInstallationDenial> {
        setup.handler::<source_control::Binding, _>(source_control::Handler)?;
        producers::parent::install(
            setup,
            configuration.selections.clone(),
            configuration.fault,
            configuration.child_width,
        )?;
        producers::child::install(
            setup,
            configuration.selections.clone(),
            configuration.fault,
            configuration.child_width,
        )?;
        producers::independent_parent::install(
            setup,
            configuration.selections.clone(),
            Fault::None,
            configuration.child_width,
        )?;
        producers::leaf::install(
            setup,
            configuration.selections,
            configuration.fault,
            configuration.child_width,
        )
    }
}
