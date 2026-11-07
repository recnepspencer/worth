//! Lawful bootstrap uses ordinary typed fields; reads use the installed View.
use super::*;
use worth_execution::ExecutionAllocationPolicy::SystemAllocation;
use worth_query_host::facade::runtime::WorthQueryExecutionRuntimeInstaller;

pub struct World {
    application: WorthQueryPrimaryGraphApplicationRuntime<Fixture>,
    invariant: WorthQueryApplicationInvariantProjectionAuthority<Fixture>,
}
type Seed = WorthQueryApplicationEntitySeed<Fixture, Item>;

fn seed_case(seed: Seed, case: u8, value: u64) -> Seed {
    macro_rules! select_case {
        ($($marker:ident: $n:literal: $name:literal),+) => {
            match case {
                $($n => seed.record_field(
                    record_field(ROOT, ITEM, $marker, "number").resolve().unwrap(), value,
                ).unwrap(),)+
                _ => panic!("fixture case is declared"),
            }
        };
    }
    each_case!(select_case)
}
impl World {
    pub fn new() -> Self {
        worth_query_host::facade::application_contribution::with_bootstrap_advancement(
            resources::execution_policy(),
            |phase| {
                let declaration = Fixture::declaration().unwrap();
                let package =
                    WorthQueryPortableDomainPackage::new(WorthQueryPortableDomainIdentity::new(
                        Fixture::OWNER,
                        Fixture::MAJOR,
                        Fixture::MINOR,
                    ))
                    .application_schema(declaration.clone())
                    .validate()
                    .unwrap();
                let admitted =
                    WorthQueryInstallationAdmissionProfile::new("support", "configuration")
                        .admit(package)
                        .unwrap();
                let (runtime, authority) = WorthQueryExecutionRuntimeInstaller::new()
                    .install(WorthQueryInstallationGeneration::initial(), [admitted])
                    .unwrap()
                    .into_parts();
                let schema = runtime
                    .installed_packages()
                    .bind_application_schema(declaration)
                    .unwrap();
                let mut bootstrap = authority
                    .prepare_primary_graph(
                        &phase,
                        &runtime,
                        &schema,
                        resources::product_world_resources(),
                    )
                    .unwrap();
                let binding = schema.principal_binding(principal::binding()).unwrap();
                bootstrap.bind_principal(
                    &binding, WorthQueryApplicationPrincipalKey::new("person").unwrap(), 1,
                    worth_query_host::facade::declaration::authentication::WorthQueryExternalPrincipalIdentity::new(
                        "https://issuer.example", "person").unwrap(),
                    worth_query_host::facade::declaration::authentication::WorthQueryPrincipalMappingStatus::Enabled,
                ).unwrap();
                for case in 1..=32 {
                    bootstrap
                        .bind_entity(seed_case(seed(case as u64), case, 1_000 + case as u64))
                        .unwrap();
                }
                // Presence must refuse before this first case's checked constructor.
                bootstrap
                    .bind_entity(seed_case(seed_case(seed(33), 1, 0), 32, 1_032))
                    .unwrap();
                let invariant = bootstrap.retain_invariant_projection_authority();
                let application = bootstrap
                    .conditional_application_runtime_installation(
                        &phase,
                        runtime,
                        authority,
                        schema,
                        SignalConditionalEvaluationBudget::development(),
                    )
                    .unwrap()
                    .publish(&phase)
                    .unwrap();
                Self {
                    application,
                    invariant,
                }
            },
        )
        .unwrap()
    }
    pub fn read<M, EP, MP>(&self, id: u64, marker: M) -> Result<M::Output, FragmentDenial>
    where
        M: In<<Fixture as Fragment>::Members, (EP, MP), (Fixture, Item)>
            + FragmentReadable<Bindings, MP, Item>,
    {
        self.application
            .with_host_advancement(|phase| {
                self.invariant
                    .project_operation::<Inspect<Form>, _>(
                        &phase,
                        |reader| {
                            let mut view = reader.fragment(ROOT).unwrap();
                            let item = view.resolve(
                                record_field(ROOT, ITEM, Identity, "id").resolve().unwrap(),
                                id,
                            )?;
                            let mut admission = |_, _| Ok(());
                            view.read(&item, marker, &mut admission)
                        },
                        SystemAllocation,
                    )
                    .unwrap()
                    .into_output()
            })
            .unwrap()
    }
}
fn seed(id: u64) -> Seed {
    Seed::new(
        entity(ROOT, ITEM),
        WorthQueryApplicationEntityKey::new(format!("entry-{id}")).unwrap(),
    )
    .record_field(
        record_field(ROOT, ITEM, Identity, "id").resolve().unwrap(),
        id,
    )
    .unwrap()
}
