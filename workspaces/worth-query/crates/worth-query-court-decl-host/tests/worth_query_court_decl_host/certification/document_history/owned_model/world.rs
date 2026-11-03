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
        setup.handler::<ownership_change::Binding, _>(ownership_change::Handler)?;
        setup.handler::<generation::Binding, _>(generation::Handler)?;
        setup.handler::<edit::member::Binding, _>(edit::member::Handler)
    }
}
pub fn install() -> Host {
    install_with_shared_descendant(false)
}
pub fn install_with_shared_descendant(shared: bool) -> Host {
    in_memory_program(ApplicationProgramAuthoring::<Schema, Program>::begin().validated_program().unwrap(),
        Schema::declaration().unwrap(), ((),),
        ApplicationBuildIdentity::parse("91820000-0000-0000-0000-000000005355").unwrap(),
        host_limits(worth_query_host::facade::admission::authenticated_principal::WorthQueryAdmissionClock::system()),
        |_, graph, installed| {
            graph.bind_principal(&installed.principal_binding(Actor::reference()).unwrap(),
                WorthQueryApplicationPrincipalKey::new("operator").unwrap(), 1,
                operator_identity(), WorthQueryPrincipalMappingStatus::Enabled)?;
            for key in ["a", "b"] {
                graph.bind_entity(WorthQueryApplicationEntitySeed::new(Model::reference(),
                    WorthQueryApplicationEntityKey::new(key).unwrap())
                    .field(ModelKey::reference(), key.to_owned()).field(ModelValue::reference(), 1))?;
            }
            for (key, value) in [("direct", 11), ("nested", 22), ("foreign", 33), ("reference", 44)] {
                graph.bind_entity(WorthQueryApplicationEntitySeed::new(Member::reference(),
                    WorthQueryApplicationEntityKey::new(key).unwrap())
                    .field(MemberKey::reference(), key.to_owned()).field(MemberValue::reference(), value))?;
            }
            for (model, member) in [("a", "direct"), ("b", "foreign")] {
                graph.bind_relation(WorthQueryApplicationRelationSeed::new(ModelMember::reference(),
                    format!("owns:{model}"), WorthQueryApplicationEntityKey::new(model).unwrap(),
                    WorthQueryApplicationEntityKey::new(member).unwrap()))?;
            }
            graph.bind_relation(WorthQueryApplicationRelationSeed::new(Child::reference(), "nested-link",
                WorthQueryApplicationEntityKey::new("direct").unwrap(), WorthQueryApplicationEntityKey::new("nested").unwrap()))?;
            graph.bind_relation(WorthQueryApplicationRelationSeed::new(Reference::reference(), "reference-link",
                WorthQueryApplicationEntityKey::new("a").unwrap(), WorthQueryApplicationEntityKey::new("reference").unwrap()))?;
            if shared {
                graph.bind_relation(WorthQueryApplicationRelationSeed::new(ModelMember::reference(), "shared-owner",
                    WorthQueryApplicationEntityKey::new("a").unwrap(), WorthQueryApplicationEntityKey::new("reference").unwrap()))?;
                graph.bind_relation(WorthQueryApplicationRelationSeed::new(Child::reference(), "second-path",
                    WorthQueryApplicationEntityKey::new("reference").unwrap(), WorthQueryApplicationEntityKey::new("nested").unwrap()))?;
            }
            Ok(())
        }).unwrap()
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
pub fn member_value(
    host: &Host,
    branch: worth_query_host::facade::product::WorthQueryProductBranch,
    key: &str,
) -> u64 {
    let scope = request_scope();
    let principal = authenticate_operator(host.installed_schema(), &scope);
    host.request(&principal, &scope)
        .on_branch(branch)
        .query(query::member::Read(key.into()))
        .execute()
        .unwrap()
        .rows()[0]
        .value
}

/// A description for independent comparison; it never admits an edit or owner.
pub fn member_identity(
    host: &Host,
    branch: worth_query_host::facade::product::WorthQueryProductBranch,
    key: &str,
) -> (u32, u64, u32) {
    let scope = request_scope();
    host.with_host_advancement(|phase| {
        let selected = host.on_branch(branch).select(&phase).unwrap();
        let id = selected
            .resolve_entity(
                &phase,
                MemberKey::reference(),
                key.to_owned(),
                &scope,
                WorthQueryPrincipalResolutionMode::Certification,
            )
            .unwrap()
            .relational_record_identity_parts();
        (id.partition_id(), id.local_slot(), id.generation())
    })
    .unwrap()
}
