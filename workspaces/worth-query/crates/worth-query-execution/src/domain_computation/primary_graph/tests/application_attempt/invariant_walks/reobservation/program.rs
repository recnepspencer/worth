//! One actual admitted proposal creates or removes a group member.
use super::{
    super::{program::authorization::authorize, schema::*, world::World},
    declaration::ChangeNode,
};
use crate::domain_computation::primary_graph::*;
use worth_execution::ExecutionAllocationPolicy as Allocation;
use worth_query_admission::facade::authenticated_principal::WorthQueryRequestScope;

#[derive(Clone, Copy)]
pub(super) enum Change {
    Create,
    Delete,
}

pub(super) fn change_node(
    world: &World,
    request: &WorthQueryRequestScope,
    change: Change,
) -> WorthQueryApplicationEffectProgram<WalkSchema, ChangeNode, ReplaceNextInput, Node> {
    world
        .application
        .with_host_advancement(|phase| {
            let host = phase.test_host_request();
            let execution = host
                .execution_request_for(world.application.product_runtime())
                .unwrap();
            let (admission, root) = authorize(
                world,
                &phase,
                &execution,
                request,
                0,
                ChangeNode::reference(),
                None,
            );
            let (_, projection, _) = world
                .invariant
                .project_admitted_operation(
                    &phase,
                    &admission,
                    |reader, root| prepare_scope(reader, root, change),
                    Allocation::SystemAllocation,
                )
                .unwrap()
                .into_parts();
            let mut reads = world
                .application
                .begin_projected_application_read_attempt(
                    &phase,
                    admission,
                    projection,
                    Allocation::SystemAllocation,
                )
                .unwrap();
            let selected = world
                .application
                .select_product_branch(
                    &execution,
                    world.application.product_runtime().default_branch(),
                )
                .unwrap();
            let member = match change {
                Change::Create => None,
                Change::Delete => Some(
                    selected
                        .resolve_entity(
                            &phase,
                            NodeKey::reference(),
                            6,
                            request,
                            WorthQueryPrincipalResolutionMode::Ordinary,
                        )
                        .unwrap(),
                ),
            };
            let old_link = member.as_ref().map(|member| {
                reads
                    .observe_relation(&execution, GroupRoot::reference(), member, &root)
                    .unwrap()
            });
            let mut effects = reads
                .complete_projected_dependencies(&phase, Allocation::SystemAllocation)
                .unwrap()
                .begin_effect_program();
            let root = effects.existing_entity(&root).unwrap();
            match (change, member, old_link) {
                (Change::Create, None, None) => {
                    create_member(&mut effects, &root);
                }
                (Change::Delete, Some(member), Some(link)) => {
                    assert_eq!(link.count(), 1, "the real owned membership must exist");
                    effects.unlink(GroupRoot::reference(), link).unwrap();
                    let member = effects.existing_entity(&member).unwrap();
                    effects.delete_entity(Node::reference(), &member).unwrap();
                }
                _ => unreachable!("the selected change owns its exact member/link inputs"),
            }
            effects.finish().unwrap()
        })
        .unwrap()
}

fn prepare_scope(
    reader: &mut WorthQueryApplicationOperationInvariantProjectionReader<
        '_,
        '_,
        WalkSchema,
        ChangeNode,
    >,
    root: &WorthQueryInvariantEntityIdentity<WalkSchema, Node>,
    change: Change,
) {
    reader
        .require_decision_entity(root, Node::reference())
        .unwrap();
    reader
        .require_decision_field(root, NodeKey::reference())
        .unwrap();
    let member = reader
        .resolve_optional_entity(NodeKey::reference(), 6)
        .unwrap();
    match change {
        Change::Create => assert!(member.is_none()),
        Change::Delete => {
            let member = member.unwrap();
            reader
                .require_decision_entity(&member, Node::reference())
                .unwrap();
            reader
                .require_decision_field(&member, NodeKey::reference())
                .unwrap();
            reader
                .require_decision_relation(GroupRoot::reference(), &member, root)
                .unwrap();
        }
    }
}

fn create_member(
    effects: &mut WorthQueryApplicationEffectProgramBuilder<
        WalkSchema,
        ChangeNode,
        ReplaceNextInput,
        Node,
    >,
    root: &WorthQueryApplicationEffectEntity<WalkSchema, Node>,
) {
    let member = effects
        .create_entity_in_context(
            root,
            Node::reference(),
            WorthQueryApplicationEntityKey::new("reobservation-node-6").unwrap(),
        )
        .unwrap();
    effects
        .initialize_field(&member, NodeKey::reference(), 6)
        .unwrap();
    effects
        .initialize_field(&member, Active::reference(), false)
        .unwrap();
    effects
        .initialize_field(&member, Blocked::reference(), false)
        .unwrap();
    effects
        .link(
            GroupRoot::reference(),
            "reobservation-group-6",
            &member,
            root,
        )
        .unwrap();
}
