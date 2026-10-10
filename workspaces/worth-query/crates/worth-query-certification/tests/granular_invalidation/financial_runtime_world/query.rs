use std::sync::Arc;

use worth_query::facade::runtime::WorthQueryPrimaryGraphSourceAdapter;
use worth_query::facade::{domain, read, runtime};

use super::contract::{FinancialDomain, FinancialFamily, FinancialOperation};
use super::query_domain::{self, FinancialExecutor, FinancialQueryProfile};
use super::query_source::FinancialSourceProjection;

#[path = "query/installation.rs"]
mod installation;
#[path = "query/providers.rs"]
mod providers;

use providers::{ConditionalCompute, EligibleProvider, QueryQuoteComparator};

pub type FinancialGraph = crate::query_runtime_world::PrimaryGraph;

pub struct FinancialQueryWorld {
    pub workspace: runtime::WorthQueryWorkspace,
    pub live: domain::WorthQueryLiveBoundDomainProjection<
        FinancialDomain,
        FinancialOperation,
        FinancialFamily,
        worth_query::facade::foundation::ObservationLaneWitness,
    >,
    pub collection: Option<domain::WorthQueryCollectionConsumerWindow>,
}

pub type FinancialSharedLease = domain::WorthQuerySharedLiveProjectionLease<
    FinancialDomain,
    FinancialOperation,
    FinancialFamily,
    worth_query::facade::foundation::ObservationLaneWitness,
>;

pub struct SharedFinancialQueryWorld {
    pub workspace: runtime::WorthQueryWorkspace,
    pub subject: FinancialSharedLease,
    pub candidate: FinancialSharedLease,
}

pub fn build_curve(
    request_execution: &worth_execution::ExecutionRequest<'_, '_>,
    host: &super::host::FinancialCourtroomWorld,
) -> FinancialQueryWorld {
    build(
        request_execution,
        host,
        FinancialQueryProfile::CurveRisk,
        false,
        None,
        0,
    )
}

pub fn build_sibling_curve_record(
    request_execution: &worth_execution::ExecutionRequest<'_, '_>,
    host: &super::host::FinancialCourtroomWorld,
) -> FinancialQueryWorld {
    build(
        request_execution,
        host,
        FinancialQueryProfile::CurveRecordRisk,
        false,
        Some(host.sibling_curve_record_identity()),
        0,
    )
}

pub fn build_quote(
    request_execution: &worth_execution::ExecutionRequest<'_, '_>,
    host: &super::host::FinancialCourtroomWorld,
) -> FinancialQueryWorld {
    build(
        request_execution,
        host,
        FinancialQueryProfile::QuoteRisk,
        false,
        None,
        0,
    )
}

pub fn build_portfolio_with_unrelated_rows(
    request_execution: &worth_execution::ExecutionRequest<'_, '_>,
    host: &super::host::FinancialCourtroomWorld,
    unrelated_rows: usize,
) -> FinancialQueryWorld {
    build(
        request_execution,
        host,
        FinancialQueryProfile::OrderedPortfolio,
        false,
        None,
        unrelated_rows,
    )
}

pub fn build_shared_curve(
    request_execution: &worth_execution::ExecutionRequest<'_, '_>,
    host: &super::host::FinancialCourtroomWorld,
) -> SharedFinancialQueryWorld {
    let FinancialQueryWorld {
        mut workspace,
        live,
        collection: _,
    } = build(
        request_execution,
        host,
        FinancialQueryProfile::CurveRisk,
        true,
        None,
        0,
    );
    let candidate = settle(&mut workspace, FinancialQueryProfile::CurveRisk).into_lifecycle();
    let shared = match live.share_with(candidate, &mut workspace) {
        domain::WorthQueryProjectionSharingOutcome::Shared(shared) => shared,
        domain::WorthQueryProjectionSharingOutcome::Stopped(stop) => {
            panic!("financial projection sharing stopped: {}", stop.detail())
        }
    };
    let (subject, candidate) = shared.into_leases();
    subject
        .admit_consumer_delivery_policy(
            &mut workspace,
            domain::WorthQuerySharedConsumerDeliveryPolicy::new(
                "desk-risk-monitoring",
                "desk-risk-public",
                "desk-risk-cursor",
                runtime::DeliveryBackpressurePolicy::RetainWithinWindow,
            )
            .unwrap(),
        )
        .unwrap();
    candidate
        .admit_consumer_delivery_policy(
            &mut workspace,
            domain::WorthQuerySharedConsumerDeliveryPolicy::new(
                "regulatory-capital",
                "restricted-capital",
                "regulatory-cursor",
                runtime::DeliveryBackpressurePolicy::DropWithGapNotice,
            )
            .unwrap(),
        )
        .unwrap();
    SharedFinancialQueryWorld {
        workspace,
        subject,
        candidate,
    }
}

#[path = "query/build.rs"]
mod build;
use build::build;

fn settle(
    workspace: &mut runtime::WorthQueryWorkspace,
    profile: FinancialQueryProfile,
) -> domain::WorthQuerySettledDomainProjection<
    FinancialDomain,
    FinancialOperation,
    FinancialFamily,
    worth_query::facade::foundation::ObservationLaneWitness,
> {
    let installed = workspace.domain(FinancialDomain).unwrap();
    let bound = workspace
        .observe_operating_world(workspace.current_world())
        .unwrap()
        .family(FinancialFamily)
        .bind(&installed, FinancialOperation)
        .unwrap();
    let consumer = bound.consumer_projection_contract().unwrap();
    let published = bound
        .admit_execution_resources((), query_domain::resource_request(), &*workspace)
        .unwrap()
        .execute(workspace)
        .unwrap()
        .publish()
        .unwrap();
    if profile == FinancialQueryProfile::OrderedPortfolio {
        let mut request = consumer.projection_request();
        request
            .select_derived_native_field_name("PortfolioValueField")
            .unwrap();
        request
            .select_derived_native_field_name("PortfolioDeskField")
            .unwrap();
        published
            .consume_bound(request.build().unwrap())
            .unwrap()
            .settle()
            .unwrap()
    } else {
        published
            .consume(
                consumer,
                read::project_facts().entity_identities().display_field(
                    read::ProjectionFactFieldPath::from_canonical_field_path(
                        worth_foundational::facade::CanonicalFieldPath::new(vec![
                            worth_foundational::facade::FieldKey::new("RiskFacts").unwrap(),
                            worth_foundational::facade::FieldKey::new("RiskValueField").unwrap(),
                        ])
                        .unwrap(),
                    ),
                ),
            )
            .unwrap()
            .settle()
            .unwrap()
    }
}
