#![allow(
    dead_code,
    reason = "Gate 1 retains graph-owned appearance fact selectors for later consumers"
)]

use std::collections::BTreeMap;

use crate::capability::{CapabilitySnapshot, ComponentId};
use crate::declaration::{UiAspectName, UiAspectSemanticSlice};
use crate::fact_contract::{UiAuthoredFactSelector, UiConsumedFactContract, UiProducedFact};
use crate::graph::UiGraphSnapshot;

use super::{
    UiAuthoredDeclarationLookup, UiGraphFactConsumerIdentity, UiGraphFactConsumerKey,
    UiGraphFactConsumerKind, UiGraphFactConsumptionRelation, UiGraphFactIndexBasis,
    UiGraphFactIndexEntry, UiGraphFactLookupDenial, UiGraphFactLookupReceipt,
};

mod appearance_consumer_contract;
mod appearance_slot_relation;
mod appearance_state;
mod authored_aspect_consumers;
mod canonical_entries;
mod consumer;
mod projection_consumer;
mod subsystem;

use super::intent_posture::intent_posture_consumers;
use appearance_consumer_contract::UiGraphAppearanceConsumerContract;
use appearance_slot_relation::UiGraphAppearanceThemeSlotDomain;
use canonical_entries::canonical_entries;
use consumer::{consumer_identity, consumer_key};
use subsystem::{build_subsystem_index, UiGraphSubsystemFactIndex};

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct UiGraphConsumedFactIndex {
    basis: UiGraphFactIndexBasis,
    appearance_consumers: UiGraphAppearanceConsumerContract,
    appearance_theme_slots: UiGraphAppearanceThemeSlotDomain,
    authored_by_declaration: BTreeMap<Box<str>, Box<[UiGraphFactIndexEntry]>>,
    query_by_projection:
        BTreeMap<worth_ui_query_binding::WorthUiQueryViewIdentity, Box<[UiGraphFactIndexEntry]>>,
    intent_posture_by_node:
        BTreeMap<crate::graph::UiGraphNodeIdentity, Box<[UiGraphFactIndexEntry]>>,
    subsystem: UiGraphSubsystemFactIndex,
}

impl UiGraphConsumedFactIndex {
    pub(crate) fn rebuild(
        snapshot: &UiGraphSnapshot,
        capabilities: &CapabilitySnapshot,
        authored_declarations: &UiAuthoredDeclarationLookup,
        projection_contents: &[crate::runtime::WorthUiProjectionContentEdge],
    ) -> Self {
        let mut authored_by_declaration =
            direct_authored_consumers(snapshot, authored_declarations);
        authored_aspect_consumers::add_authored_aspect_consumers(
            snapshot,
            authored_declarations,
            &mut authored_by_declaration,
        );
        add_static_paint_token_consumers(
            snapshot,
            capabilities,
            authored_declarations,
            &mut authored_by_declaration,
        );
        appearance_slot_relation::add_role_slot_consumers(
            snapshot,
            capabilities,
            authored_declarations,
            &mut authored_by_declaration,
        );
        let appearance_consumers =
            UiGraphAppearanceConsumerContract::from_graph(snapshot, capabilities);
        let appearance_theme_slots = UiGraphAppearanceThemeSlotDomain::from_capabilities(
            capabilities,
            authored_declarations,
        );
        let authored_by_declaration = authored_by_declaration
            .into_iter()
            .map(|(identity, entries)| (identity, canonical_entries(entries)))
            .collect();

        Self {
            basis: UiGraphFactIndexBasis::from_generation(snapshot, capabilities),
            appearance_consumers,
            appearance_theme_slots,
            authored_by_declaration,
            query_by_projection: query_projection_consumers(snapshot, projection_contents),
            intent_posture_by_node: intent_posture_consumers(snapshot),
            subsystem: build_subsystem_index(snapshot),
        }
    }

    pub(crate) const fn appearance_axis_demand(
        &self,
    ) -> crate::runtime::appearance::UiAppearanceStateAxisDemand {
        self.appearance_consumers.axis_demand()
    }

    pub(crate) const fn has_appearance_consumers(&self) -> bool {
        self.appearance_consumers.has_consumers()
    }

    pub(crate) fn appearance_state_consumer_nodes(
        &self,
        axis: worth_ui_dsl::UiAppearanceStateAxis,
    ) -> Box<[crate::graph::UiGraphNodeIdentity]> {
        self.appearance_consumers.state_consumer_nodes(axis).into()
    }

    pub(crate) fn appearance_role_consumer_nodes(
        &self,
        role: &worth_ui_dsl::UiAppearanceRoleIdentity,
    ) -> Box<[crate::graph::UiGraphNodeIdentity]> {
        self.appearance_consumers.role_consumers(role).into()
    }

    pub(crate) fn appearance_attached_consumer_nodes(
        &self,
    ) -> Box<[crate::graph::UiGraphNodeIdentity]> {
        self.appearance_consumers.attached_consumer_nodes()
    }

    pub(crate) fn appearance_required_role_identities(
        &self,
    ) -> Box<[worth_ui_dsl::UiAppearanceRoleIdentity]> {
        self.appearance_consumers.required_role_identities()
    }

    pub(crate) fn has_same_appearance_consumer_contract(&self, other: &Self) -> bool {
        self.appearance_consumers == other.appearance_consumers
    }

    pub(crate) const fn basis(&self) -> UiGraphFactIndexBasis {
        self.basis
    }

    pub(crate) fn lookup_retained(
        &self,
        fact: &UiProducedFact,
    ) -> Result<UiGraphFactLookupReceipt, UiGraphFactLookupDenial> {
        self.lookup(self.basis, fact)
    }

    pub(crate) fn lookup(
        &self,
        requested_basis: UiGraphFactIndexBasis,
        fact: &UiProducedFact,
    ) -> Result<UiGraphFactLookupReceipt, UiGraphFactLookupDenial> {
        if requested_basis != self.basis {
            return Err(UiGraphFactLookupDenial::BasisMismatch {
                index_basis: self.basis,
                requested_basis,
            });
        }

        let entries = match fact {
            UiProducedFact::AuthoredSource(authored) => match authored.selector() {
                UiAuthoredFactSelector::Module(_) => &[][..],
                UiAuthoredFactSelector::Node(identity) => self
                    .authored_by_declaration
                    .get(identity)
                    .map(Box::as_ref)
                    .ok_or_else(|| UiGraphFactLookupDenial::UnknownAuthoredDeclaration {
                        authored_identity: identity.clone(),
                    })?,
            },
            UiProducedFact::Query(query) => match query.projection_identity() {
                Some(identity) => self
                    .query_by_projection
                    .get(identity)
                    .map(Box::as_ref)
                    .unwrap_or_default(),
                None => self.subsystem.entries_for(fact.family()),
            },
            UiProducedFact::IntentPosture(posture) => self
                .intent_posture_by_node
                .get(&posture.graph_node())
                .map(Box::as_ref)
                .unwrap_or_default(),
            _ => self.subsystem.entries_for(fact.family()),
        };
        debug_assert!(entries
            .iter()
            .all(|entry| entry.consumed_fact_contract().matches(fact)));
        Ok(UiGraphFactLookupReceipt::new(
            self.basis,
            entries.to_vec().into_boxed_slice(),
        ))
    }
}

fn query_projection_consumers(
    snapshot: &UiGraphSnapshot,
    projection_contents: &[crate::runtime::WorthUiProjectionContentEdge],
) -> BTreeMap<worth_ui_query_binding::WorthUiQueryViewIdentity, Box<[UiGraphFactIndexEntry]>> {
    let mut by_projection = BTreeMap::<
        worth_ui_query_binding::WorthUiQueryViewIdentity,
        Vec<UiGraphFactIndexEntry>,
    >::new();
    let affected_aspect = UiAspectName::from_semantic_slice(UiAspectSemanticSlice::ContentText);
    for content in projection_contents {
        let contract =
            UiConsumedFactContract::query_projection(content.projection_identity().clone());
        let entries = by_projection
            .entry(content.projection_identity().clone())
            .or_default();
        for node in snapshot.nodes().iter().filter(|node| {
            node.declaration_identity().authored_semantic_name() == content.component_identity()
        }) {
            push_component_consumer(
                entries,
                snapshot,
                node,
                contract.clone(),
                UiGraphFactConsumptionRelation::general(Some(affected_aspect.clone())),
            );
        }
    }
    by_projection
        .into_iter()
        .map(|(identity, entries)| (identity, canonical_entries(entries)))
        .collect()
}

fn add_static_paint_token_consumers(
    snapshot: &UiGraphSnapshot,
    capabilities: &CapabilitySnapshot,
    authored_declarations: &UiAuthoredDeclarationLookup,
    by_declaration: &mut BTreeMap<Box<str>, Vec<UiGraphFactIndexEntry>>,
) {
    for node in snapshot.nodes() {
        let Some(component) =
            component_capability_for_node(node, capabilities, authored_declarations)
        else {
            continue;
        };
        let Some(static_paint) = component.static_paint_contract() else {
            continue;
        };
        let token_capability_identity = static_paint.theme_token().as_str();
        let token_identity: Box<str> = authored_declarations
            .theme_token_declaration_identity(token_capability_identity)
            .unwrap_or(token_capability_identity)
            .into();
        let contract = UiConsumedFactContract::authored(token_identity.clone());
        let affected_aspect =
            UiAspectName::from_semantic_slice(UiAspectSemanticSlice::AppearanceBackground);
        let entries = by_declaration.entry(token_identity).or_default();
        push_component_consumer(
            entries,
            snapshot,
            node,
            contract,
            UiGraphFactConsumptionRelation::static_paint(
                token_capability_identity,
                affected_aspect,
            ),
        );
    }
}

pub(super) fn component_capability_for_node<'capability>(
    node: &crate::graph::UiGraphNode,
    capabilities: &'capability CapabilitySnapshot,
    authored_declarations: &UiAuthoredDeclarationLookup,
) -> Option<&'capability crate::capability::ComponentDescriptor> {
    if let Some(identity) = node.component_reference() {
        return capabilities.components().get(identity);
    }
    let source_backed = authored_declarations
        .unique_component_capability_identity(node.authored_provenance_digest())
        .and_then(|identity| ComponentId::new(identity).ok())
        .and_then(|identity| capabilities.components().get(&identity));
    source_backed.or_else(|| {
        ComponentId::new(node.declaration_identity().authored_semantic_name())
            .ok()
            .and_then(|identity| capabilities.components().get(&identity))
    })
}

fn fact_selector_identity<'identity>(
    provenance_digest: u64,
    fallback_identity: &'identity str,
    authored_declarations: &'identity UiAuthoredDeclarationLookup,
) -> &'identity str {
    authored_declarations
        .unique_identity(provenance_digest)
        .unwrap_or(fallback_identity)
}

pub(super) fn push_component_consumer(
    entries: &mut Vec<UiGraphFactIndexEntry>,
    snapshot: &UiGraphSnapshot,
    node: &crate::graph::UiGraphNode,
    contract: UiConsumedFactContract,
    consumption_relation: UiGraphFactConsumptionRelation,
) {
    let authored_identity: Box<str> = node.declaration_identity().authored_semantic_name().into();
    let repeated = node.repeated_instance_basis().identity_digest();
    entries.push(UiGraphFactIndexEntry::new_with_relation(
        UiGraphFactConsumerKey::new(
            UiGraphFactConsumerKind::GraphNode,
            authored_identity.clone(),
            repeated,
        ),
        UiGraphFactConsumerIdentity::GraphNode(node.graph_node_identity()),
        consumption_relation.clone(),
        contract.clone(),
    ));
    if let Some(slot) = snapshot.mount_eligibility_slot_for_node(node.graph_node_identity()) {
        entries.push(UiGraphFactIndexEntry::new_with_relation(
            UiGraphFactConsumerKey::new(
                UiGraphFactConsumerKind::MountEligibilitySlot,
                authored_identity,
                repeated,
            ),
            UiGraphFactConsumerIdentity::MountEligibilitySlot(slot.mount_eligibility_identity()),
            consumption_relation,
            contract,
        ));
    }
}

fn direct_authored_consumers(
    snapshot: &UiGraphSnapshot,
    authored_declarations: &UiAuthoredDeclarationLookup,
) -> BTreeMap<Box<str>, Vec<UiGraphFactIndexEntry>> {
    let mut by_declaration = BTreeMap::<Box<str>, Vec<UiGraphFactIndexEntry>>::new();
    for node in snapshot.nodes() {
        let consumer_identity: Box<str> =
            node.declaration_identity().authored_semantic_name().into();
        let selector_identity: Box<str> = fact_selector_identity(
            node.authored_provenance_digest(),
            node.declaration_identity().authored_semantic_name(),
            authored_declarations,
        )
        .into();
        let contract = UiConsumedFactContract::authored(selector_identity.clone());
        let entries = by_declaration.entry(selector_identity).or_default();
        entries.push(UiGraphFactIndexEntry::new(
            UiGraphFactConsumerKey::new(
                UiGraphFactConsumerKind::GraphNode,
                consumer_identity.clone(),
                node.repeated_instance_basis().identity_digest(),
            ),
            UiGraphFactConsumerIdentity::GraphNode(node.graph_node_identity()),
            None,
            contract.clone(),
        ));
        if let Some(slot) = snapshot.mount_eligibility_slot_for_node(node.graph_node_identity()) {
            entries.push(UiGraphFactIndexEntry::new(
                UiGraphFactConsumerKey::new(
                    UiGraphFactConsumerKind::MountEligibilitySlot,
                    consumer_identity,
                    node.repeated_instance_basis().identity_digest(),
                ),
                UiGraphFactConsumerIdentity::MountEligibilitySlot(
                    slot.mount_eligibility_identity(),
                ),
                None,
                contract,
            ));
        }
    }
    by_declaration
}
