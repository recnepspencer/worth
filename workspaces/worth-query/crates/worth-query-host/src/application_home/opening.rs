//! The sole Host recipe consumes Store's actual new-Home opportunity once.
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
        let query =
            match query.install_home_native_schema(phase, home.native_for_schema_installation()) {
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
            finish_new_home(
                phase,
                home,
                query
                    .take()
                    .expect("the one admitted recipe consumes Query once"),
                self.initial_state,
            )
        });
        match opened {
            Ok(opened) => opened,
            Err(home) => Err(Refused::QueryPublicationReadmissionRequired {
                query: query
                    .take()
                    .expect("a refused origin never entered the recipe"),
                home,
            }),
        }
    }
}

fn finish_new_home<Schema, Program>(
    phase: &WorthQueryBootstrapAdvancementPhase<'_>,
    mut home: UnsharedReadmittedApplicationHome,
    query: PreparedHomeNativeMeaning<Schema, Program>,
    initial_state: impl FnOnce(
        &WorthQueryBootstrapAdvancementPhase<'_>,
        &mut worth_query_execution::facade::primary_graph::WorthQueryPrimaryGraphBootstrap<Schema>,
        &worth_query_installation::facade::WorthQueryInstalledApplicationSchema<Schema>,
    ) -> Result<
        (),
        worth_query_execution::facade::primary_graph::WorthQueryPrimaryGraphInstallationDenial,
    >,
) -> Result<
    WorthQueryApplicationHome<Schema, Program>,
    WorthQueryApplicationHomeOpenRefused<Schema, Program>,
>
where
    Schema: ApplicationSchemaComposition,
    Program: ApplicationProgramDefinition<Schema> + 'static,
    Program::Outputs:
        ApplicationProgramOutputsShape<Schema> + WorthQueryApplicationProgramRoots<Schema>,
{
    use WorthQueryApplicationHomeOpenRefused as Refused;
    let request = phase.execution_request();
    let (native, authority, outcome) = home.query_finalization_parts();
    let query = match query.finish_fresh_home_native_readmission(phase, native, authority, outcome)
    {
        Ok(query) => query,
        Err(query) => return Err(Refused::NativeFinalization { query, home }),
    };
    let bridge = match query.home_bridge_definition() {
        Ok(bridge) => bridge,
        Err(cause) => return Err(Refused::BridgeDefinition { query, home, cause }),
    };
    let (budgets, clock, policy, execution, conditionals) = query.home_world_resources();
    let definition = ApplicationHomeWorldDefinition::new(
        bridge,
        "primary-application-source",
        budgets,
        clock,
        policy,
        execution,
        conditionals,
    );
    let home = match readmit_query_world(home.share(), definition, &request) {
        Ok(home) => home.consume(|home| home),
        Err(home) => return Err(Refused::WorldReadmission { query, home }),
    };
    let (source, bridge, conditional) = match home.query_composition_parts() {
        Ok(parts) => parts,
        Err(cause) => return Err(Refused::Composition { query, home, cause }),
    };
    let mut query = match query.attach_home_registered_source(phase, source, bridge, conditional) {
        Ok(query) => query,
        Err(query) => return Err(Refused::SourceAttachment { query, home }),
    };
    // Only the user initializer is caught by this borrowed Query operation;
    // the actual Home and Query bootstrap remain outside its unwind boundary.
    if let Err(cause) = query.initialize_home_query_state(phase, initial_state) {
        return Err(Refused::InitialState { query, home, cause });
    }
    if let Err(cause) = query.publish_home_query_seed(phase) {
        return Err(Refused::SeedPublication { query, home, cause });
    }
    let mut home = home;
    let root = match home.select_query_root(&request) {
        Ok(root) => root,
        Err(cause) => return Err(Refused::WorldRoot { query, home, cause }),
    };
    let ports = home.world();
    let query = match query.attach_home_world(
        phase,
        ports.publication_port(),
        ports.branch_port(),
        ports.observation_port(),
        ports.inspection_port(),
        ports.recovery_port(),
        root,
        directory_requirement_refusal,
    ) {
        Ok(query) => query.consume(|query| query),
        Err(query) => return Err(Refused::QueryAttachment { query, home }),
    };
    finish_fresh_service(phase, query, home)
}

/// Private payloads are never returned before both actual admission phases.
struct HomeServiceCustody<Schema: ApplicationSchemaComposition, Program> {
    query: ReadmittedHomeQueryService<Schema, Program>,
    home: DirectoryHome,
}
impl<Schema: ApplicationSchemaComposition, Program: 'static> AuthorityMarker
    for HomeServiceCustody<Schema, Program>
{
}

struct SettledHomeService<Schema: ApplicationSchemaComposition, Program> {
    query: ReadmittedHomeQueryService<Schema, Program>,
    home: NoPendingApplicationHomeTransition,
}
impl<Schema: ApplicationSchemaComposition, Program: 'static> AuthorityMarker
    for SettledHomeService<Schema, Program>
{
}

fn finish_fresh_service<Schema: ApplicationSchemaComposition, Program: 'static>(
    phase: &WorthQueryBootstrapAdvancementPhase<'_>,
    query: ReadmittedHomeQueryService<Schema, Program>,
    home: DirectoryHome,
) -> Result<
    WorthQueryApplicationHome<Schema, Program>,
    WorthQueryApplicationHomeOpenRefused<Schema, Program>,
> {
    let readmit = (|| {
        phase
            .execution_request()
            .consult()
            .map_err(HomeCustodyReadmissionDenial::Execution)?;
        let held = home
            .world()
            .inspection_port()
            .recovery_snapshot()
            .map_err(HomeCustodyReadmissionDenial::WorldUnavailable)?;
        if held.installed() != 0
            || held.reserved() != 0
            || held.abandoned() != 0
            || held.updating() != 0
        {
            return Err(HomeCustodyReadmissionDenial::RetainedWorldCustody {
                installed: held.installed(),
                reserved: held.reserved(),
                abandoned: held.abandoned(),
                updating: held.updating(),
            });
        }
        Ok(())
    })();
    if let Err(cause) = readmit {
        return Err(WorthQueryApplicationHomeOpenRefused::CustodyReadmission {
            query,
            home,
            cause,
        });
    }
    let custody = CustodyReadmitted::from_owner(HomeServiceCustody { query, home });
    let settled = custody.consume(|HomeServiceCustody { query, home }| {
        match home.readmit_no_pending_owners(&phase.execution_request()) {
            Ok(home) => Ok(TransitionSettled::from_owner(SettledHomeService {
                query,
                home,
            })),
            Err(home) => Err(WorthQueryApplicationHomeOpenRefused::OwnerAdmission { query, home }),
        }
    })?;
    Ok(settled.consume(
        |SettledHomeService { query, home }| WorthQueryApplicationHome {
            query: query.into_home_program_runtime(),
            directory: Some(home.into_no_pending_home()),
        },
    ))
}
