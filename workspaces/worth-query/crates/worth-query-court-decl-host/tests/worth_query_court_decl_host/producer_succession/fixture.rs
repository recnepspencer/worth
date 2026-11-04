//! One source defines both genuine typed producer-tree schema profiles.
macro_rules! producer_succession_fixture {
    () => {
        #[macro_use]
        #[path = "../producer_succession/readiness.rs"]
        mod readiness;
        #[macro_use]
        #[path = "../producer_succession/retirement.rs"]
        mod retirement;
        #[path = "../producer_succession/aggregate_schema.rs"]
        mod aggregate_schema;
        #[path = "../producer_succession/correction.rs"]
        mod correction;
        #[path = "../producer_succession/producer.rs"]
        mod producer;
        #[path = "../producer_succession/producers.rs"]
        mod producers;
        #[path = "../producer_succession/program.rs"]
        mod program;
        #[path = "../producer_succession/query.rs"]
        mod query;
        #[path = "../producer_succession/realization.rs"]
        mod realization;
        #[path = "../producer_succession/realization_read.rs"]
        mod realization_read;
        #[path = "../producer_succession/schema.rs"]
        mod schema;
        #[path = "../producer_succession/settlement.rs"]
        mod settlement;
        #[path = "../producer_succession/source_control.rs"]
        mod source_control;
        #[path = "../producer_succession/world.rs"]
        mod world;
        #[path = "../producer_succession/write_fence.rs"]
        mod write_fence;
        use producer::*;
        use program::Program;
        use query::*;
        use retirement::Retirement;
        use schema::*;
        use std::sync::{Arc, Mutex};
        use worth_query_decl::facade::application_aftermath::*;
        use worth_query_decl::facade::{
            application_operation::*, application_program::*, application_query::*,
            application_schema::*, *,
        };
        use worth_query_host::facade::{
            application_contribution::*, application_entry::*,
            application_installation as installation, primary_graph::*,
        };
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
            pub read_predecessor_orbits: bool,
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
                producers::leaf::contracts(c)
                    .and_then(|()| producers::independent_parent::contracts(c))
            }
            fn configure(
                configuration: Configuration,
                setup: &mut WorthQueryApplicationContributionSetup<'_, Schema>,
            ) -> Result<(), WorthQueryPrimaryGraphInstallationDenial> {
                setup.handler::<source_control::Binding, _>(source_control::Handler)?;
                setup.handler::<write_fence::Binding, _>(write_fence::Handler)?;
                producers::parent::install(
                    setup,
                    configuration.selections.clone(),
                    configuration.fault,
                    configuration.child_width,
                    configuration.read_predecessor_orbits,
                )?;
                producers::child::install(
                    setup,
                    configuration.selections.clone(),
                    configuration.fault,
                    configuration.child_width,
                    configuration.read_predecessor_orbits,
                )?;
                producers::independent_parent::install(
                    setup,
                    configuration.selections.clone(),
                    Fault::None,
                    configuration.child_width,
                    configuration.read_predecessor_orbits,
                )?;
                producers::leaf::install(
                    setup,
                    configuration.selections,
                    configuration.fault,
                    configuration.child_width,
                    configuration.read_predecessor_orbits,
                )
            }
        }
    };
}
