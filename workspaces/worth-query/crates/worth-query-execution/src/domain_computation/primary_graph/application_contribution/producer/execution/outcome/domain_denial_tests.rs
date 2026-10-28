//! A handler's domain denial leaves the completion path as the producer's
//! declared value, decodable by its binding, under the domain-denied kind.
use super::*;
use crate::domain_computation::primary_graph::{
    WorthQueryProducerDenialDecodeDenial, WorthQueryUndeclaredProducerDenial,
};
use worth_query_declaration::facade::application_schema::ValueDecodeBounds;

#[derive(Clone, Debug, Eq, PartialEq, worth_foundational::facade::WorthValue)]
struct PlanRefusal {
    rule: u64,
    evidence: String,
}
worth_query_declaration::worth_query_value_binding!(RefusalBinding for PlanRefusal {
    identity: "tests.completion-refusal.v1", scalar: String,
    canonical: ValueDecodeBounds::new(96, 64, 4, 4),
});

fn refused<Declared>() -> WorthQueryOutputDemandDenial
where
    Declared:
        crate::domain_computation::primary_graph::WorthQueryProducerDenialDeclaration<PlanRefusal>,
{
    completed_handler::<(), _, Declared>(
        "the producer",
        HandlerResult::DomainDenied(PlanRefusal {
            rule: 3,
            evidence: "edge 4 crosses edge 9".into(),
        }),
        |_| Some("planar refusal"),
    )
    .unwrap_err()
}

#[test]
fn a_declared_domain_denial_decodes_from_the_completed_handler() {
    let denial = refused::<RefusalBinding>();
    assert_eq!(
        denial.kind(),
        WorthQueryOutputDemandDenialKind::ProducerDomainDenied
    );
    assert_eq!(
        denial.decode_producer_denial::<RefusalBinding>(),
        Ok(PlanRefusal {
            rule: 3,
            evidence: "edge 4 crosses edge 9".into(),
        })
    );
}

#[test]
fn an_undeclared_producer_still_denies_on_the_domain_but_carries_no_value() {
    let denial = refused::<WorthQueryUndeclaredProducerDenial>();
    assert_eq!(
        denial.kind(),
        WorthQueryOutputDemandDenialKind::ProducerDomainDenied
    );
    assert_eq!(
        denial.decode_producer_denial::<RefusalBinding>(),
        Err(WorthQueryProducerDenialDecodeDenial::Undeclared)
    );
}
