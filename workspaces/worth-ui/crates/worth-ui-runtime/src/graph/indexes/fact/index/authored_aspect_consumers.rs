use std::collections::BTreeMap;

use crate::fact_contract::UiConsumedFactContract;
use crate::graph::{UiGraphAspectPublisherKind, UiGraphSnapshot};

use super::{
    consumer_identity, consumer_key, fact_selector_identity, UiAuthoredDeclarationLookup,
    UiGraphFactIndexEntry,
};

pub(super) fn add_authored_aspect_consumers(
    snapshot: &UiGraphSnapshot,
    authored_declarations: &UiAuthoredDeclarationLookup,
    by_declaration: &mut BTreeMap<Box<str>, Vec<UiGraphFactIndexEntry>>,
) {
    let indexes = snapshot.core_indexes();
    for (aspect, publishers) in indexes.published_aspects().iter() {
        for publisher in publishers {
            let UiGraphAspectPublisherKind::GraphNode(publisher_node) = publisher.kind() else {
                continue;
            };
            let publisher_lookup = snapshot
                .lookup()
                .graph_node(publisher_node)
                .expect("every published graph node remains indexed");
            let publisher = publisher_lookup.value();
            let selector_identity: Box<str> = fact_selector_identity(
                publisher.authored_provenance_digest(),
                publisher.declaration_identity().authored_semantic_name(),
                authored_declarations,
            )
            .into();
            let contract = UiConsumedFactContract::authored(selector_identity.clone());
            let entries = by_declaration.entry(selector_identity).or_default();
            for consumer in indexes.consumed_aspects().consumers_for(aspect) {
                entries.push(UiGraphFactIndexEntry::new(
                    consumer_key(snapshot, consumer.kind()),
                    consumer_identity(consumer.kind()),
                    Some(aspect.clone()),
                    contract.clone(),
                ));
            }
        }
    }
}
