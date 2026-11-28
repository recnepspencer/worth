use super::*;
pub mod home;
pub use home::Runtime;
pub type Host = WorthQueryProgramApplicationRuntime<Schema, Program>;
impl WorthQueryApplicationContribution<Schema> for Contribution {
    type Configuration = ();
    fn contracts(
        contracts: &mut WorthQueryApplicationContributionContracts<Schema>,
    ) -> Result<(), WorthQueryPrimaryGraphInstallationDenial> {
        coupled::producer::contracts(contracts)
    }
    fn configure(
        _: (),
        setup: &mut WorthQueryApplicationContributionSetup<'_, Schema>,
    ) -> Result<(), WorthQueryPrimaryGraphInstallationDenial> {
        setup.handler::<edit::model::Binding, _>(edit::model::Handler)?;
        setup.handler::<delete::Binding, _>(delete::Handler)?;
        setup.handler::<probe::Binding, _>(probe::Handler)?;
        setup.handler::<refusals::Binding, _>(refusals::Handler)?;
        setup.handler::<outputs::sculpt::Binding, _>(outputs::sculpt::Handler)?;
        setup.handler::<stored::Binding, _>(stored::Handler)?;
        setup.handler::<retarget::Binding, _>(retarget::Handler)?;
        setup.handler::<groups::source_currentness::Binding, _>(
            groups::source_currentness::Handler,
        )?;
        coupled::producer::install(setup)?;
        invariant_names::install(setup)?;
        setup.handler::<edit::member::Binding, _>(edit::member::Handler)
    }
}
pub fn install() -> Host {
    try_install(true).unwrap()
}
/// Installs the world; `named` false seeds members without their names.
pub fn try_install(named: bool) -> Result<Host, String> {
    try_install_program::<Program>(named, "91820000-0000-0000-0000-00000000a71e")
}
/// Installs the seeded world on program `P` under build identity `build`.
pub fn try_install_program<P>(
    named: bool,
    build: &str,
) -> Result<WorthQueryProgramApplicationRuntime<Schema, P>, String>
where
    P: ApplicationProgramDefinition<Schema, Contributions = (Contribution,)>,
    P::Outputs: WorthQueryApplicationProgramRoots<Schema>,
{
    let limits = host_limits(
        worth_query_host::facade::admission::authenticated_principal::WorthQueryAdmissionClock::system(),
    );
    in_memory_program(
        ApplicationProgramAuthoring::<Schema, P>::begin()
            .validated_program()
            .unwrap(),
        Schema::declaration().unwrap(),
        ((),),
        ApplicationBuildIdentity::parse(build).unwrap(),
        limits,
        |_, graph, installed| seed(graph, installed, named),
    )
    .map_err(|cause| format!("{cause:?}"))
}
/// Seeds the operator, models `a` and `b`, members (named when `named`)
/// and their links: the one initial state every installation starts from.
pub fn seed(
    graph: &mut WorthQueryPrimaryGraphBootstrap<Schema>,
    installed: &worth_query_host::facade::domain::WorthQueryInstalledApplicationSchema<Schema>,
    named: bool,
) -> Result<(), WorthQueryPrimaryGraphInstallationDenial> {
    graph.bind_principal(
        &installed.principal_binding(Actor::reference()).unwrap(),
        WorthQueryApplicationPrincipalKey::new("operator").unwrap(),
        1,
        operator_identity(),
        WorthQueryPrincipalMappingStatus::Enabled,
    )?;
    for key in ["a", "b"] {
        graph.bind_entity(
            WorthQueryApplicationEntitySeed::new(
                Model::reference(),
                WorthQueryApplicationEntityKey::new(key).unwrap(),
            )
            .field(ModelKey::at("ModelKey"), key.to_owned())
            .field(ModelValue::at("ModelValue"), 1),
        )?;
    }
    for (key, value) in [
        ("direct", 11),
        ("nested", 22),
        ("foreign", 33),
        ("reference", 44),
        ("loose", 55),
    ] {
        let seed = WorthQueryApplicationEntitySeed::new(
            Member::reference(),
            WorthQueryApplicationEntityKey::new(key).unwrap(),
        )
        .field(MemberKey::at("MemberKey"), key.to_owned())
        .field(MemberValue::at("MemberValue"), value);
        let seed = if named {
            seed.persistent_name("authored")
        } else {
            seed
        };
        graph.bind_entity(seed)?;
    }
    for (model, member) in [("a", "direct"), ("b", "foreign")] {
        graph.bind_relation(WorthQueryApplicationRelationSeed::new(
            ModelMember::reference(),
            format!("owns:{model}"),
            WorthQueryApplicationEntityKey::new(model).unwrap(),
            WorthQueryApplicationEntityKey::new(member).unwrap(),
        ))?;
    }
    graph.bind_relation(WorthQueryApplicationRelationSeed::new(
        Child::reference(),
        "nested-link",
        WorthQueryApplicationEntityKey::new("direct").unwrap(),
        WorthQueryApplicationEntityKey::new("nested").unwrap(),
    ))?;
    graph.bind_relation(WorthQueryApplicationRelationSeed::new(
        Reference::binding(),
        "reference-link",
        WorthQueryApplicationEntityKey::new("a").unwrap(),
        WorthQueryApplicationEntityKey::new("reference").unwrap(),
    ))?;
    graph.bind_relation(WorthQueryApplicationRelationSeed::new(
        Reference::binding(),
        "foreign-loose-reference",
        WorthQueryApplicationEntityKey::new("b").unwrap(),
        WorthQueryApplicationEntityKey::new("loose").unwrap(),
    ))?;
    Ok(())
}
/// A description for independent comparison; it never admits an edit or owner.
pub fn member_identity<P>(
    host: &WorthQueryProgramApplicationRuntime<Schema, P>,
    branch: worth_query_host::facade::product::WorthQueryProductBranch,
    key: &str,
) -> Option<(u32, u64, u32)>
where
    P: ApplicationProgramDefinition<Schema>,
{
    let scope = request_scope();
    host.with_host_advancement(|phase| {
        let selected = host.on_branch(branch).select(&phase).unwrap();
        selected
            .resolve_entity(
                &phase,
                MemberKey::at("MemberKey"),
                key.to_owned(),
                &scope,
                WorthQueryPrincipalResolutionMode::Certification,
            )
            .ok()
            .map(|entity| {
                let id = entity.relational_record_identity_parts();
                (id.partition_id(), id.local_slot(), id.generation())
            })
    })
    .unwrap()
}
