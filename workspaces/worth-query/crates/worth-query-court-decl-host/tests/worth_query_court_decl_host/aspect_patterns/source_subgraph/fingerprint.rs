//! Descriptive content identity; native observations still decide legality.
use super::*;
#[derive(WorthValue)]
#[worth_value(crate = "worth_query_decl::facade::application_schema")]
struct Content {
    format: String,
    node_fields: String,
    membership: String,
    graph: GraphRow,
}
worth_query_value_binding!(ContentBinding for Content {
    identity: "source-subgraph.content.v1", scalar: String,
    canonical: ValueDecodeBounds::new(4096, 256, 16, 16),
});
pub fn content(graph: &GraphRow) -> String {
    let value = Content {
        format: "source-subgraph.content.v1".into(),
        node_fields: "Source/SourceFacts:SourceKey=String,SourceValue=UInt64".into(),
        membership: "SourceMember:root->members;fixed-star;unique-Key-order".into(),
        graph: graph.clone(),
    };
    let encoded = ContentBinding::encode(&value).unwrap();
    let text = StringApplicationValueBinding::decode(&encoded).unwrap();
    // One canonical frame is one hash part, so separator bytes in keys cannot
    // create ambiguous part boundaries. The existing digest owner owns SHA-256.
    worth_query_decl::facade::identity::hash_parts(&[text])
}
