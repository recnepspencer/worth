//! Normal installation, authenticated mutation, and native current-format recovery.
pub use super::{operation, query, schema::*};
pub use worth_query_testkit_document_retention::facade::document_retention_model::operator_identity::*;
pub use worth_query_host::facade::application_installation::ApplicationBuildIdentity;
pub use worth_query_host::facade::{
    application_contribution::*, application_entry::*, application_installation::*,
    declaration::application_program::*, declaration::authentication::*, primary_graph::*,
};

pub struct Program;
pub struct Feature;
impl ApplicationFeature<AdmissionSchema> for Feature {
    type Inputs = ApplicationFeatureInputLeaf;
    const IDENTITY: &'static str = "canonical-admission.feature.v1";
}
impl ApplicationProgramDefinition<AdmissionSchema> for Program {
    type Contributions = (AdmissionContribution,);
    type Outputs = ApplicationProgramOutputs<ApplicationNoOutputGraph>;
    type Rules = ApplicationRuleLeaf;
    const IDENTITY: ApplicationProgramIdentity =
        ApplicationProgramIdentity::new("canonical-admission.program.v1");
    fn feature_specs() -> Vec<ApplicationFeatureSpec> {
        vec![ApplicationFeatureSpec::root::<AdmissionSchema, Feature>()
            .native_action::<operation::AdvanceBinding>()
            .finish_action()
            .native_action::<super::live::publish::PublishBinding>()
            .finish_action()
            .finish()]
    }
}
impl WorthQueryApplicationContribution<AdmissionSchema> for AdmissionContribution {
    type Configuration = ();
    fn configure(
        _: (),
        setup: &mut WorthQueryApplicationContributionSetup<'_, AdmissionSchema>,
    ) -> Result<(), WorthQueryPrimaryGraphInstallationDenial> {
        setup.handler::<operation::AdvanceBinding, _>(operation::Handler)?;
        setup.handler::<super::live::publish::PublishBinding, _>(super::live::publish::Handler)
    }
}
pub struct Host {
    pub program: WorthQueryProgramApplicationRuntime<AdmissionSchema, Program>,
    pub request_work: u64,
}
pub fn install(work: u64) -> Host {
    install_owned(work, false).0
}
pub fn install_page(work: u64) -> Host {
    install_owned(work, true).0
}
pub fn install_with_projection(
    work: u64,
) -> (
    Host,
    WorthQueryApplicationInvariantProjectionAuthority<AdmissionSchema>,
) {
    install_owned(work, false)
}
pub fn install_owned(
    work: u64,
    page: bool,
) -> (
    Host,
    WorthQueryApplicationInvariantProjectionAuthority<AdmissionSchema>,
) {
    let mut projection = None;
    let host = in_memory_program(
        ApplicationProgramAuthoring::<AdmissionSchema, Program>::begin()
            .validated_program()
            .unwrap(),
        AdmissionSchema::declaration().unwrap(),
        ((),),
        ApplicationBuildIdentity::parse("11111111-1111-1111-1111-111111111111").unwrap(),
        super::limits::limits(work.max(super::FUNDED_WORK)),
        |_, graph, installed| {
            projection = Some(graph.retain_invariant_projection_authority());
            graph.bind_principal(
                &installed
                    .principal_binding(DocumentPrincipalBinding::reference())
                    .unwrap(),
                WorthQueryApplicationPrincipalKey::new("canonical-admission-operator").unwrap(),
                1_u64,
                operator_identity(),
                WorthQueryPrincipalMappingStatus::Enabled,
            )?;
            for identity in ["root", "child"] {
                graph.bind_entity(
                    WorthQueryApplicationEntitySeed::new(Document::reference(), key(identity))
                        .field(DocumentIdentityField::reference(), identity.to_owned())
                        .field(
                            DocumentValueField::reference(),
                            super::opaque::SwallowingValue(super::value()),
                        )
                        .field(CounterField::reference(), 0_u64),
                )?;
            }
            graph.bind_relation(WorthQueryApplicationRelationSeed::new(
                ChildLink::reference(),
                "canonical-admission-child",
                key("root"),
                key("child"),
            ))?;
            if page {
                graph.bind_relation(WorthQueryApplicationRelationSeed::new(
                    ChildLink::reference(),
                    "canonical-admission-self",
                    key("root"),
                    key("root"),
                ))?;
            }
            Ok(())
        },
    )
    .expect("ordinary canonical admission schema installs");
    let projection = projection.expect("real bootstrap retains its native projection authority");
    (
        Host {
            program: host,
            request_work: work,
        },
        projection,
    )
}

pub fn key(identity: &str) -> WorthQueryApplicationEntityKey<AdmissionSchema, Document> {
    WorthQueryApplicationEntityKey::new(identity).unwrap()
}
pub fn restore(checkpoint: WorthQueryApplicationCheckpoint, work: u64) -> Host {
    let program = in_memory_program_from_checkpoint(
        ApplicationProgramAuthoring::<AdmissionSchema, Program>::begin()
            .validated_program()
            .unwrap(),
        AdmissionSchema::declaration().unwrap(),
        ((),),
        ApplicationBuildIdentity::parse("11111111-1111-1111-1111-111111111111").unwrap(),
        super::limits::limits(work.max(super::FUNDED_WORK)),
        checkpoint,
    )
    .expect("current source restores without authored conversion");
    Host {
        program,
        request_work: work,
    }
}
pub fn capture(host: &Host) -> WorthQueryApplicationCheckpoint {
    host.program
        .runtime()
        .with_host_advancement(|phase| {
            host.program
                .runtime()
                .capture_application_checkpoint(
                    &phase,
                    WorthQueryCheckpointCapturePolicy::SystemAllocation,
                )
                .unwrap()
        })
        .unwrap()
}
pub fn advance(
    host: &Host,
    reads: operation::Reads,
    idempotency: u64,
) -> Result<
    WorthQueryApplicationMutationOutcome<operation::Denied, operation::Written>,
    WorthQueryApplicationRequestMutationDenial,
> {
    operation::reset();
    super::opaque::reset();
    let scope = request_scope();
    let principal = authenticate_operator(host.program.installed_schema(), &scope);
    super::request_funding::with_request(host, &scope, |phase| {
        host.program
            .runtime()
            .request(&principal, &scope)
            .on_branch(host.program.current_world())
            .advanced()
            .mutate(operation::Intent {
                input: operation::Input { reads },
            })
            .without_source()
            .idempotency(&idempotency)
            .execute_in_program_in_advancement(&phase, &host.program)
    })
}
pub fn read(host: &Host, identity: &str) -> query::Row {
    let scope = request_scope();
    let principal = authenticate_operator(host.program.installed_schema(), &scope);
    super::request_funding::with_request(host, &scope, |phase| {
        host.program
            .runtime()
            .request(&principal, &scope)
            .on_branch(host.program.current_world())
            .query(query::ReadValue {
                identity: identity.to_owned(),
            })
            .execute_in_advancement(&phase)
    })
    .expect("ordinary query reads native source")
    .rows()[0]
        .clone()
}

pub fn read_repeated(host: &Host) -> Result<query::Row, WorthQueryApplicationRequestQueryDenial> {
    super::repeated_query::reset();
    super::opaque::reset();
    let scope = request_scope();
    let principal = authenticate_operator(host.program.installed_schema(), &scope);
    super::request_funding::with_request(host, &scope, |phase| {
        host.program
            .runtime()
            .request(&principal, &scope)
            .on_branch(host.program.current_world())
            .query(super::repeated_query::ReadRepeated {
                identity: "root".to_owned(),
            })
            .execute_in_advancement(&phase)
    })
    .map(|result| result.rows()[0].clone())
}
