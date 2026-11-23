use super::*;
pub type Host = WorthQueryProgramApplicationRuntime<Schema, Program>;
pub type Document<'a> = WorthQueryApplicationHistoryDocument<'a, Schema, History>;
impl WorthQueryApplicationContribution<Schema> for Contribution {
    type Configuration = ();
    fn configure(
        _: (),
        setup: &mut WorthQueryApplicationContributionSetup<'_, Schema>,
    ) -> Result<(), WorthQueryPrimaryGraphInstallationDenial> {
        setup.handler::<edit::model::Binding, _>(edit::model::Handler)?;
        setup.handler::<delete::Binding, _>(delete::Handler)?;
        setup.handler::<probe::Binding, _>(probe::Handler)?;
        setup.handler::<refusals::Binding, _>(refusals::Handler)?;
        setup.handler::<outputs::sculpt::Binding, _>(outputs::sculpt::Handler)?;
        setup.handler::<edit::member::Binding, _>(edit::member::Handler)
    }
}
pub fn install() -> Host {
    try_install(true).unwrap()
}
/// Installs the world; `named` false seeds members without their names.
pub fn try_install(named: bool) -> Result<Host, String> {
    let limits = host_limits(
        worth_query_host::facade::admission::authenticated_principal::WorthQueryAdmissionClock::system(),
    );
    in_memory_program(
        ApplicationProgramAuthoring::<Schema, Program>::begin()
            .validated_program()
            .unwrap(),
        Schema::declaration().unwrap(),
        ((),),
        ApplicationBuildIdentity::parse("91820000-0000-0000-0000-00000000a71e").unwrap(),
        limits,
        |_, graph, installed| {
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
            navigation::seed(graph)?;
            Ok(())
        },
    )
    .map_err(|cause| format!("{cause:?}"))
}
pub fn attach(host: &Host) -> Document<'_> {
    attach_as::<History>(host)
}
pub fn attach_as<H>(host: &Host) -> WorthQueryApplicationHistoryDocument<'_, Schema, H>
where
    H: ApplicationHistoryBinding<Schema, Access = query::model::Read>,
{
    let scope = request_scope();
    let principal = authenticate_operator(host.installed_schema(), &scope);
    host.with_host_advancement(|phase| {
        let selected = host.on_branch(host.current_world()).select(&phase).unwrap();
        let head = host
            .branches()
            .fork_expected(selected)
            .unwrap()
            .components(|parts| parts.fork_relational().reuse_exact_signal_basis())
            .create(&phase)
            .unwrap()
            .into_owned_head();
        host.request(&principal, &scope)
            .on_branch(head.branch())
            .attach_history::<H>(&phase, head, query::model::Read("a".into()))
            .unwrap()
    })
    .unwrap()
}
/// A description for independent comparison; it never admits an edit or owner.
pub fn member_identity(
    host: &Host,
    branch: worth_query_host::facade::product::WorthQueryProductBranch,
    key: &str,
) -> Option<(u32, u64, u32)> {
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
