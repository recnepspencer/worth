//! An entity's advertised reverse reads are exactly the queries
//! `referenced_by` admits: scoped on the entity, every root path starting
//! with a reverse step into it along a declared incoming relation.
use super::*;
use worth_query_declaration::facade::application_schema::RelationDirection;

fn reverse<'q>(relation: &'q str, from: &'q str) -> Option<FirstStep<'q>> {
    Some(FirstStep {
        direction: RelationDirection::Reverse,
        relation,
        from,
        to: "Document",
    })
}

fn forward<'q>(relation: &'q str, to: &'q str) -> Option<FirstStep<'q>> {
    Some(FirstStep {
        direction: RelationDirection::Forward,
        relation,
        from: "Document",
        to,
    })
}

/// A Document with two declared incoming relations and no other members.
fn document() -> WorthQueryEntityImpact {
    let mut impact = WorthQueryEntityImpact::in_members(&[], "Document");
    for (relation, from) in [("Cites", "Memo"), ("Governs", "Grant")] {
        impact.incoming.push(WorthQueryIncomingRelation {
            edge: WorthQueryRelationEdge::new(relation, from, "Document"),
            reverse_reads: Vec::new(),
        });
    }
    impact
}

fn reads(impact: &WorthQueryEntityImpact, relation: &str) -> Vec<String> {
    impact
        .incoming()
        .iter()
        .find(|incoming| incoming.edge().relation() == relation)
        .map(|incoming| incoming.reverse_reads().to_vec())
        .unwrap_or_default()
}

#[test]
fn only_queries_referenced_by_admits_are_advertised_as_reverse_reads() {
    let mut impact = document();
    impact.attach_query("citing_memos", "Document", [reverse("Cites", "Memo")]);
    impact.attach_query(
        "all_referrers",
        "Document",
        [reverse("Governs", "Grant"), reverse("Cites", "Memo")],
    );
    // Multi-hop: the reverse step into Document is not the first step and the
    // query is scoped on another entity.
    impact.attach_query("grant_memos", "Grant", [reverse("Cites", "Memo")]);
    // Mixed: one root path reads an incoming link, another an outgoing one.
    impact.attach_query(
        "mixed",
        "Document",
        [reverse("Cites", "Memo"), forward("Owns", "Page")],
    );
    // A root-less query reads its scoped record itself.
    impact.attach_query("self_read", "Document", Vec::new());
    impact.attach_query("empty_path", "Document", [None]);
    // Repeating an admitted query never duplicates it.
    impact.attach_query("citing_memos", "Document", [reverse("Cites", "Memo")]);

    assert_eq!(reads(&impact, "Cites"), ["citing_memos", "all_referrers"]);
    assert_eq!(reads(&impact, "Governs"), ["all_referrers"]);
}

#[test]
fn the_shared_rule_returns_sorted_distinct_incoming_relations() {
    let read = super::super::reverse_read::incoming_relations_read(
        "Document",
        [
            reverse("Governs", "Grant"),
            reverse("Cites", "Memo"),
            reverse("Governs", "Grant"),
        ],
    );
    assert_eq!(
        read,
        Some(vec![
            WorthQueryRelationEdge::new("Cites", "Memo", "Document"),
            WorthQueryRelationEdge::new("Governs", "Grant", "Document"),
        ])
    );
    let into_other = Some(FirstStep {
        direction: RelationDirection::Reverse,
        relation: "Cites",
        from: "Memo",
        to: "Page",
    });
    assert_eq!(
        super::super::reverse_read::incoming_relations_read("Document", [into_other]),
        None
    );
}
