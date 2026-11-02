//! Route only the actual Store origin into its closed Home recipe.
use super::{refusal::*, request::*, runtime::*, support::directory_requirement_refusal};
use worth_proof::{AuthorityMarker, CustodyReadmitted, TransitionSettled};
use worth_query_declaration::facade::{application_program::*, application_schema::*};
use worth_query_execution::facade::{
    application_contribution::{
        WorthQueryApplicationContributionTuple, WorthQueryBootstrapAdvancementPhase,
    },
    application_home_integration::*,
    application_installation::*,
};
use worth_store_application::*;

impl<'authoring, Schema, Program> WorthQueryApplicationHomeRequest<'authoring, Schema, Program>
where
    Schema: ApplicationSchemaComposition,
    Program: ApplicationProgramDefinition<Schema> + 'static,
    Program::Outputs:
        ApplicationProgramOutputsShape<Schema> + WorthQueryApplicationProgramRoots<Schema>,
    Program::Contributions: WorthQueryApplicationContributionTuple<Schema>,
{
    pub fn open_in_advancement(
        self,
        phase: &WorthQueryBootstrapAdvancementPhase<'_>,
    ) -> Result<
        WorthQueryApplicationHome<Schema, Program>,
        WorthQueryApplicationHomeOpenRefused<Schema, Program>,
    > {
        use WorthQueryApplicationHomeOpenRefused as Refused;
        let location = match self.location {
            WorthQueryApplicationHomeLocation::MemoryOnly => {
                return in_memory_rostered_program_in_advancement(
                    phase,
                    self.program,
                    self.roster,
                    self.declaration,
                    self.configuration,
                    self.build,
                    self.limits,
                    self.initial_state,
                )
                .map(|query| WorthQueryApplicationHome {
                    query,
                    directory: None,
                    checkpoint_limits: self.checkpoint_limits,
                })
                .map_err(Refused::Preparation);
            }
            WorthQueryApplicationHomeLocation::Directory(location) => location,
        };
        let query = prepare_home_program(
            self.program,
            self.roster,
            self.declaration,
            self.configuration,
            self.build,
            self.limits,
        )
        .map_err(Refused::Preparation)?;
        let binding = match query.admitted_home_binding() {
            Ok(binding) => binding,
            Err(cause) => return Err(Refused::Binding { query, cause }),
        };
        let request = phase.execution_request();
        let physical = ApplicationHomeRequest::new(
            location,
            query.home_relational_definition(),
            binding,
            self.build,
        );
        let home =
            match recover_home(physical, &request).and_then(|home| admit_home(home, &request)) {
                Ok(home) => home,
                Err(cause) => return Err(Refused::Physical { query, cause }),
            };
        let mut home = match attach_relational(home, &request) {
            Ok(home) => home,
            Err(home) => return Err(Refused::RelationalAttachment { query, home }),
        };
        let installed = match home.prepare_native_schema_for_home(&request) {
            Ok(ApplicationHomeNativeSchemaPreparation::Fresh {
                native,
                initialization,
            }) => query.initialize_home_native_schema(phase, native, initialization),
            Ok(ApplicationHomeNativeSchemaPreparation::Existing { native }) => {
                query.prepare_home_native_receiver_schema(phase, native)
            }
            Err(cause) => Err(query.refuse_home_native_schema(cause)),
        };
        let query = match installed {
            Ok(query) => query,
            Err(query) => return Err(Refused::NativeInstallation { query, home }),
        };
        let home = match home.readmit(query.home_initial_schema_receipt(), &request) {
            Ok(home) => home.consume(|home| home),
            Err(home) => return Err(Refused::NativeReadmission { query, home }),
        };
        // Existing/Consumed origin never invokes the callback. Keep Query out
        // of its closure's ownership until Store has actually admitted it.
        let mut query = Some(query);
        let opened = home.initialize_new_home(&request, |home| {
            fresh::finish_new_home(
                phase,
                home,
                query
                    .take()
                    .expect("the one admitted recipe consumes Query once"),
                self.initial_state,
                self.checkpoint_limits,
            )
        });
        match opened {
            Ok(opened) => opened,
            Err(home) => match home.reason() {
                ApplicationHomeInitializationDenial::ExistingHomeRequiresQueryPublication => {
                    existing::finish_existing_home(
                        phase,
                        home.into_home(),
                        query.take().expect("existing origin retained Query"),
                        self.checkpoint_limits,
                    )
                }
                ApplicationHomeInitializationDenial::Execution(_)
                | ApplicationHomeInitializationDenial::OpportunityAlreadyConsumed => {
                    Err(Refused::QueryPublicationReadmissionRequired {
                        query: query.take().expect("refused origin retained Query"),
                        home,
                    })
                }
            },
        }
    }
}

mod existing;
mod fresh;
mod service;
