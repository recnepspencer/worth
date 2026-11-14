//! Repeated first-family history has both predecessor and separate foreign inputs.
use super::super::super::hierarchy::program as hierarchy;
use super::super::super::program as base;
use super::*;
use occurrence_topology as topology;
pub struct Previous;
impl ApplicationConnectionIdentity for Previous {
    const IDENTITY: &'static str = "interleaved.actual-predecessors";
}
impl ApplicationOccurrenceConnectionBinding<Schema, hierarchy::Parent, hierarchy::Parent>
    for Previous
{
}
pub type Connection = ApplicationConnectionRef<
    Schema,
    hierarchy::Parent,
    hierarchy::Output,
    hierarchy::Parent,
    hierarchy::PredecessorInput,
    Previous,
>;
pub type Tree =
    ApplicationOutputGraph<program::ModelConnection, ApplicationOutputOccurrences<(Connection,)>>;
impl WorthQueryApplicationDependentOutputConnection<Schema> for Previous {
    type RootDemand = first::Demand;
    type Demand = first::Demand;
    type Discovery = topology::Read;
    const IDENTITY: &'static str = <Self as ApplicationConnectionIdentity>::IDENTITY;
    fn discovery_from_root(
        root: &first::Demand,
    ) -> Result<topology::Read, WorthQueryRequiredOutputConnectionDenial> {
        Ok(topology::Read(root.0.clone()))
    }
    fn demands_from_discovery(
        found: &topology::Topology,
    ) -> Result<Vec<first::Demand>, WorthQueryRequiredOutputConnectionDenial> {
        Ok(found.children.iter().cloned().map(first::Demand).collect())
    }
}
impl WorthQueryApplicationPrerequisiteOutputConnection<Schema> for Previous {
    type Prerequisites = topology::Read;
    type PrerequisiteRow = topology::ParentBinding;
    type Selection = ApplicationQueryResultRelationRef<
        topology::Query,
        topology::Parents,
        Schema,
        ModelMember,
        Model,
        Member,
        ForwardResultTraversal,
        ManyResults,
    >;
    fn prerequisite_rows() -> Self::Selection {
        topology::parents()
    }
    fn prerequisites_from_target(
        target: &first::Demand,
    ) -> Result<topology::Read, WorthQueryRequiredOutputConnectionDenial> {
        Ok(topology::Read(target.0.clone()))
    }
    fn parent_from_prerequisite(
        row: &topology::Parent,
    ) -> Result<first::Demand, WorthQueryRequiredOutputConnectionDenial> {
        Ok(first::Demand(row.key.clone()))
    }
}
pub struct Program;
impl ApplicationProgramDefinition<Schema> for Program {
    type Contributions = (Contribution,);
    type Outputs = ApplicationProgramOutputs<(Tree, (program::ProducedTree, program::SealTree))>;
    type Rules = ApplicationRuleLeaf;
    const IDENTITY: ApplicationProgramIdentity =
        ApplicationProgramIdentity::new("owned-outputs.occurrence-history");
    fn feature_specs() -> Vec<ApplicationFeatureSpec> {
        let mut specs = program::Program::feature_specs();
        specs[0] = base::envelope_actions::<
            fixture::Delivery<Tree>,
            Completion,
            fixture::Delivery<Tree>,
            Completion,
            Completion,
            Completion,
            Completion,
        >(base::entry_builder::<Completion, Completion, Completion>())
        .action::<created::Binding, Completion>()
        .finish_action()
        .finish();
        specs
    }
}
pub(super) fn seed(
    graph: &mut WorthQueryPrimaryGraphBootstrap<Schema>,
) -> Result<(), WorthQueryPrimaryGraphInstallationDenial> {
    for (key, value) in [
        ("occ:a", 1),
        ("occ:b", 2),
        ("occ:c", 1),
        ("occ:d", 4),
        ("cycle:a", 1),
        ("cycle:b", 2),
        ("missing", 1),
    ] {
        graph.bind_entity(
            WorthQueryApplicationEntitySeed::new(
                Model::reference(),
                WorthQueryApplicationEntityKey::new(key).unwrap(),
            )
            .field(ModelKey::at("ModelKey"), key.to_owned())
            .field(ModelValue::at("ModelValue"), value),
        )?;
    }
    for (child, role, parent) in [
        ("occ:b", "previous", Some("occ:a")),
        ("occ:c", "previous", Some("occ:b")),
        ("occ:c", "foreign", Some("occ:d")),
        ("cycle:a", "previous", Some("cycle:b")),
        ("cycle:b", "previous", Some("cycle:a")),
        ("missing", "missing", None),
    ] {
        let descriptor = format!("{child}:{role}");
        graph.bind_entity(
            WorthQueryApplicationEntitySeed::new(
                Member::reference(),
                WorthQueryApplicationEntityKey::new(&descriptor).unwrap(),
            )
            .field(MemberKey::at("MemberKey"), descriptor.clone())
            .field(MemberValue::at("MemberValue"), 0),
        )?;
        graph.bind_relation(WorthQueryApplicationRelationSeed::new(
            ModelMember::reference(),
            format!("owns:{descriptor}"),
            WorthQueryApplicationEntityKey::new(child).unwrap(),
            WorthQueryApplicationEntityKey::new(&descriptor).unwrap(),
        ))?;
        if let Some(parent) = parent {
            graph.bind_relation(WorthQueryApplicationRelationSeed::new(
                UpstreamOf::reference(),
                format!("uses:{descriptor}"),
                WorthQueryApplicationEntityKey::new(&descriptor).unwrap(),
                WorthQueryApplicationEntityKey::new(parent).unwrap(),
            ))?;
        }
    }
    Ok(())
}
#[cfg(test)]
mod courts {
    use super::*;
    use worth_query_host::facade::{
        application_entry::WorthQueryEditEnvelopeOutcome as Outcome,
        runtime::ExecutionAllocationPolicy,
    };
    use worth_relational::facade::inspection::observe_maintenance_work;
    fn host() -> WorthQueryProgramApplicationRuntime<Schema, Program> {
        world::install_configured(host_limits(worth_query_host::facade::admission::authenticated_principal::WorthQueryAdmissionClock::system()),1,None,
            world::Configuration { envelope_records:true,interleaved_records:true,..Default::default() })
    }
    #[test]
    fn repeated_family_closes_previous_and_foreign_parents_before_real_source_projection() {
        let host = host();
        let scope = request_scope();
        let principal = authenticate_operator(host.installed_schema(), &scope);
        let request = host.request(&principal, &scope);
        let mut edit = request
            .edit_envelope()
            .idempotency("real-occurrence-history".to_owned())
            .runtime_stages(&host)
            .ordered_authored()
            .unwrap();
        edit.push_stage::<Tree, _>(fixture::model::Intent {
            key: "occ:a".into(),
            value: 3,
            key_derived_target: false,
        })
        .unwrap();
        let before = super::super::super::courts::native_head(&host).0;
        let contacts = host.producer_contacts_on_this_thread_for_test();
        let (outcome, writes) = observe_maintenance_work(24, || {
            edit.execute_performed(ExecutionAllocationPolicy::SystemAllocation)
                .unwrap()
        });
        let Outcome::Committed(receipt) = outcome else {
            panic!("{outcome:?}")
        };
        assert_eq!(receipt.stage_count(), 1);
        assert_eq!(writes.published_rows.len() + writes.omitted_publications, 1);
        assert_eq!(super::super::super::courts::native_head(&host).1, [before]);
        for (key, expected) in [("a", 30), ("d", 40), ("b", 320), ("c", 3610)] {
            let rows = request
                .query(query::first_output::Read(format!("first:occ:{key}")))
                .execute()
                .unwrap();
            assert_eq!(
                rows.rows()[0].value,
                expected,
                "actual predecessor/profile output was not supplied"
            );
        }
        assert_eq!(
            host.producer_contacts_on_this_thread_for_test() - contacts,
            4,
            "each actual route executes once; shared parents coalesce"
        );
    }
    #[test]
    fn actual_occurrence_cycle_and_missing_parent_refuse_before_any_supplier() {
        for key in ["cycle:a", "missing"] {
            let host = host();
            let scope = request_scope();
            let principal = authenticate_operator(host.installed_schema(), &scope);
            let request = host.request(&principal, &scope);
            let mut edit = request
                .edit_envelope()
                .idempotency(format!("refuse:{key}"))
                .runtime_stages(&host)
                .ordered_authored()
                .unwrap();
            edit.push_stage::<Tree, _>(fixture::model::Intent {
                key: key.into(),
                value: 3,
                key_derived_target: false,
            })
            .unwrap();
            let before = super::super::super::courts::native_head(&host);
            let contacts = host.producer_contacts_on_this_thread_for_test();
            let (outcome, writes) = observe_maintenance_work(16, || {
                edit.execute_performed(ExecutionAllocationPolicy::SystemAllocation)
                    .unwrap()
            });
            let Outcome::Uncommitted(WorthQueryApplicationUncommitted::Denied(cause)) = outcome
            else {
                panic!("{outcome:?}")
            };
            let refusal = cause
                .coupled_producer_denial()
                .expect("public output owner preserves the actual refusal");
            if key == "cycle:a" {
                assert!(
                    matches!(
                        refusal.kind(),
                        WorthQueryOutputDemandDenialKind::CyclicRequiredOutputOccurrences { .. }
                    ),
                    "{refusal:?}"
                );
            }
            assert_eq!(host.producer_contacts_on_this_thread_for_test(), contacts);
            assert_eq!(writes.published_rows.len() + writes.omitted_publications, 0);
            assert_eq!(super::super::super::courts::native_head(&host), before);
        }
    }
}
