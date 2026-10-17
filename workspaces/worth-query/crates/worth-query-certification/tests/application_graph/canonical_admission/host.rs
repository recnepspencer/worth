//! Normal installation, authenticated mutation, and native current-format recovery.
use super::{operation, query, schema::*};
use crate::document_retention_model::operator_identity::*;
use worth_query_host::facade::{
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
pub type Host = WorthQueryProgramApplicationRuntime<AdmissionSchema, Program>;
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
fn install_owned(
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
        super::limits::limits(work),
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
    (
        host,
        projection.expect("real bootstrap retains its native projection authority"),
    )
}
fn key(identity: &str) -> WorthQueryApplicationEntityKey<AdmissionSchema, Document> {
    WorthQueryApplicationEntityKey::new(identity).unwrap()
}
pub fn restore(checkpoint: WorthQueryApplicationCheckpoint, work: u64) -> Host {
    in_memory_program_from_checkpoint(
        ApplicationProgramAuthoring::<AdmissionSchema, Program>::begin()
            .validated_program()
            .unwrap(),
        AdmissionSchema::declaration().unwrap(),
        ((),),
        super::limits::limits(work),
        checkpoint,
    )
    .expect("current source restores without authored conversion")
}
pub fn capture(host: &Host) -> WorthQueryApplicationCheckpoint {
    host.runtime()
        .with_host_advancement(|phase| {
            host.runtime()
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
    let principal = authenticate_operator(host.installed_schema(), &scope);
    host.runtime()
        .request(&principal, &scope)
        .on_branch(host.current_world())
        .advanced()
        .mutate(operation::Intent {
            input: operation::Input { reads },
        })
        .without_source()
        .idempotency(&idempotency)
        .execute_in_program(
            host,
            worth_query_host::facade::runtime::ExecutionAllocationPolicy::SystemAllocation,
        )
}
pub fn read(host: &Host, identity: &str) -> query::Row {
    let scope = request_scope();
    let principal = authenticate_operator(host.installed_schema(), &scope);
    host.runtime()
        .request(&principal, &scope)
        .on_branch(host.current_world())
        .query(query::ReadValue {
            identity: identity.to_owned(),
        })
        .execute()
        .expect("ordinary query reads native source")
        .rows()[0]
        .clone()
}

pub fn read_repeated(host: &Host) -> Result<query::Row, WorthQueryApplicationRequestQueryDenial> {
    super::repeated_query::reset();
    super::opaque::reset();
    let scope = request_scope();
    let principal = authenticate_operator(host.installed_schema(), &scope);
    host.runtime()
        .request(&principal, &scope)
        .on_branch(host.current_world())
        .query(super::repeated_query::ReadRepeated {
            identity: "root".to_owned(),
        })
        .execute()
        .map(|result| result.rows()[0].clone())
}
