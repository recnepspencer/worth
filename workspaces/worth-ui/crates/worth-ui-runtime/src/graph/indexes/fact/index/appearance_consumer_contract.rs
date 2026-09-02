#![allow(
    dead_code,
    reason = "Gate 1 retains graph-owned appearance consumer selection facts for later consumers"
)]

use std::cmp::Ordering;
use std::collections::BTreeMap;

use crate::capability::CapabilitySnapshot;
use crate::declaration::{UiAppearanceRoleAttachment, UiDeclarationIdentity};
use crate::graph::{UiGraphSnapshot, UiRepeatedInstanceBasis};
use crate::runtime::appearance::UiAppearanceStateConsumer;

#[derive(Clone, Debug, Eq, PartialEq)]
pub(super) struct UiGraphAppearanceConsumerContract {
    has_consumers: bool,
    axis_demand: crate::runtime::appearance::UiAppearanceStateAxisDemand,
    attachments: Box<[UiGraphAppearanceAttachment]>,
    roles: Box<[worth_ui_dsl::UiAppearanceRoleDeclaration]>,
    state_consumers: [Box<[UiAppearanceStateConsumer]>; 6],
    state_consumer_nodes:
        BTreeMap<worth_ui_dsl::UiAppearanceStateAxis, Box<[crate::graph::UiGraphNodeIdentity]>>,
    role_consumers:
        BTreeMap<worth_ui_dsl::UiAppearanceRoleIdentity, Box<[crate::graph::UiGraphNodeIdentity]>>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
struct UiGraphAppearanceAttachment {
    graph_node: crate::graph::UiGraphNodeIdentity,
    declaration: UiDeclarationIdentity,
    repeated_instance: UiRepeatedInstanceBasis,
    attachment: UiAppearanceRoleAttachment,
}

impl UiGraphAppearanceConsumerContract {
    pub(super) fn from_graph(
        snapshot: &UiGraphSnapshot,
        capabilities: &CapabilitySnapshot,
    ) -> Self {
        let mut has_consumers = false;
        let mut axis_demand = crate::runtime::appearance::UiAppearanceStateAxisDemand::default();
        let mut attachments = Vec::new();
        let mut roles = Vec::new();
        let mut state_consumers: [Vec<UiAppearanceStateConsumer>; 6] =
            std::array::from_fn(|_| Vec::new());
        let mut state_consumer_nodes = BTreeMap::<
            worth_ui_dsl::UiAppearanceStateAxis,
            Vec<crate::graph::UiGraphNodeIdentity>,
        >::new();
        let mut role_consumers = BTreeMap::<
            worth_ui_dsl::UiAppearanceRoleIdentity,
            Vec<crate::graph::UiGraphNodeIdentity>,
        >::new();
        for node in snapshot.nodes() {
            let Some(attachment) = node.appearance_role_attachment() else {
                continue;
            };
            if node.component_reference() != Some(attachment.target()) {
                continue;
            }
            let Some(role) = capabilities.appearance_roles().get(attachment.role()) else {
                continue;
            };
            if role.aspect_contract() != attachment.aspect_contract()
                || role.revision() != attachment.revision()
            {
                continue;
            }
            let state_consumer =
                UiAppearanceStateConsumer::from_role(node.graph_node_identity(), role);
            has_consumers = true;
            attachments.push(UiGraphAppearanceAttachment {
                graph_node: node.graph_node_identity(),
                declaration: node.declaration_identity().clone(),
                repeated_instance: node.repeated_instance_basis().clone(),
                attachment: attachment.clone(),
            });
            if !roles
                .iter()
                .any(|admitted: &worth_ui_dsl::UiAppearanceRoleDeclaration| {
                    admitted.role() == role.role()
                })
            {
                roles.push(role.clone());
            }
            append_state_consumer(&mut axis_demand, &mut state_consumers, &state_consumer);
            role_consumers
                .entry(role.role().clone())
                .or_default()
                .push(node.graph_node_identity());
            for (_, partition) in role.partitions() {
                for axis in partition.axes() {
                    axis_demand.include(axis.axis());
                    state_consumer_nodes
                        .entry(axis.axis())
                        .or_default()
                        .push(node.graph_node_identity());
                }
            }
        }
        attachments.sort_by(compare_attachments);
        roles.sort_by(|left, right| left.role().cmp(right.role()));
        let state_consumers = state_consumers.map(|mut consumers| {
            consumers.sort_by(|left, right| {
                left.graph_node()
                    .cmp(&right.graph_node())
                    .then_with(|| left.role().cmp(right.role()))
            });
            consumers.into_boxed_slice()
        });
        canonicalize_consumers(&mut state_consumer_nodes);
        canonicalize_consumers(&mut role_consumers);
        Self {
            has_consumers,
            axis_demand,
            attachments: attachments.into_boxed_slice(),
            roles: roles.into_boxed_slice(),
            state_consumers,
            state_consumer_nodes: state_consumer_nodes
                .into_iter()
                .map(|(axis, nodes)| (axis, nodes.into_boxed_slice()))
                .collect(),
            role_consumers: role_consumers
                .into_iter()
                .map(|(role, nodes)| (role, nodes.into_boxed_slice()))
                .collect(),
        }
    }

    pub(super) const fn axis_demand(
        &self,
    ) -> crate::runtime::appearance::UiAppearanceStateAxisDemand {
        self.axis_demand
    }

    pub(super) const fn has_consumers(&self) -> bool {
        self.has_consumers
    }

    pub(super) fn state_consumers(
        &self,
        axis: worth_ui_dsl::UiAppearanceStateAxis,
    ) -> &[UiAppearanceStateConsumer] {
        &self.state_consumers[crate::runtime::appearance::UiAppearanceStateAxisDemand::index(axis)]
    }

    pub(super) fn state_consumer_nodes(
        &self,
        axis: worth_ui_dsl::UiAppearanceStateAxis,
    ) -> &[crate::graph::UiGraphNodeIdentity] {
        self.state_consumer_nodes
            .get(&axis)
            .map_or(&[], Box::as_ref)
    }

    pub(super) fn role_consumers(
        &self,
        role: &worth_ui_dsl::UiAppearanceRoleIdentity,
    ) -> &[crate::graph::UiGraphNodeIdentity] {
        self.role_consumers.get(role).map_or(&[], Box::as_ref)
    }
}

fn append_state_consumer(
    axis_demand: &mut crate::runtime::appearance::UiAppearanceStateAxisDemand,
    state_consumers: &mut [Vec<UiAppearanceStateConsumer>; 6],
    state_consumer: &UiAppearanceStateConsumer,
) {
    for axis in [
        worth_ui_dsl::UiAppearanceStateAxis::Operability,
        worth_ui_dsl::UiAppearanceStateAxis::Focus,
        worth_ui_dsl::UiAppearanceStateAxis::Validation,
        worth_ui_dsl::UiAppearanceStateAxis::Selection,
        worth_ui_dsl::UiAppearanceStateAxis::Hover,
        worth_ui_dsl::UiAppearanceStateAxis::Pressed,
    ] {
        if !state_consumer.consumes(axis) {
            continue;
        }
        axis_demand.include(axis);
        let consumers = &mut state_consumers
            [crate::runtime::appearance::UiAppearanceStateAxisDemand::index(axis)];
        if !consumers.contains(state_consumer) {
            consumers.push(state_consumer.clone());
        }
    }
}

fn canonicalize_consumers<K>(consumers: &mut BTreeMap<K, Vec<crate::graph::UiGraphNodeIdentity>>)
where
    K: Ord,
{
    for nodes in consumers.values_mut() {
        nodes.sort_unstable();
        nodes.dedup();
    }
}

fn compare_attachments(
    left: &UiGraphAppearanceAttachment,
    right: &UiGraphAppearanceAttachment,
) -> Ordering {
    left.declaration
        .authored_semantic_name()
        .cmp(right.declaration.authored_semantic_name())
        .then_with(|| {
            left.declaration
                .digest()
                .raw()
                .cmp(&right.declaration.digest().raw())
        })
        .then_with(|| compare_repeated_instances(&left.repeated_instance, &right.repeated_instance))
        .then_with(|| left.graph_node.cmp(&right.graph_node))
}

fn compare_repeated_instances(
    left: &UiRepeatedInstanceBasis,
    right: &UiRepeatedInstanceBasis,
) -> Ordering {
    use UiRepeatedInstanceBasis::{DeclarationKeyed, Denied, RuntimeDataKeyed, Unavailable};

    match (left, right) {
        (
            DeclarationKeyed {
                declaration_identity_digest: left,
            },
            DeclarationKeyed {
                declaration_identity_digest: right,
            },
        ) => left.raw().cmp(&right.raw()),
        (
            RuntimeDataKeyed {
                runtime_data_key: left,
            },
            RuntimeDataKeyed {
                runtime_data_key: right,
            },
        ) => left.as_str().cmp(right.as_str()),
        (Denied { denial: left }, Denied { denial: right }) => {
            repeated_instance_denial_order(left).cmp(&repeated_instance_denial_order(right))
        }
        (Unavailable, Unavailable) => Ordering::Equal,
        _ => repeated_instance_kind_order(left).cmp(&repeated_instance_kind_order(right)),
    }
}

fn repeated_instance_kind_order(basis: &UiRepeatedInstanceBasis) -> u8 {
    match basis {
        UiRepeatedInstanceBasis::DeclarationKeyed { .. } => 0,
        UiRepeatedInstanceBasis::RuntimeDataKeyed { .. } => 1,
        UiRepeatedInstanceBasis::Denied { .. } => 2,
        UiRepeatedInstanceBasis::Unavailable => 3,
    }
}

fn repeated_instance_denial_order(denial: &crate::graph::UiRepeatedInstanceBasisDenial) -> u8 {
    match denial {
        crate::graph::UiRepeatedInstanceBasisDenial::MissingBasis => 0,
        crate::graph::UiRepeatedInstanceBasisDenial::BasisFreeRuntimeIdentityDenied => 1,
        crate::graph::UiRepeatedInstanceBasisDenial::PositionBasedBasis => 2,
        crate::graph::UiRepeatedInstanceBasisDenial::ContradictoryBasis => 3,
    }
}
