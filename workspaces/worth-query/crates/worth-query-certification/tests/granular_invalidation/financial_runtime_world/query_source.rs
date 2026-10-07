use std::collections::BTreeMap;

use worth_foundational::facade::{
    AspectValue, CanonicalFieldPath, ContractValidatedAspectValueView, FieldKey,
};
use worth_query::facade::runtime::WorthQueryPrimaryGraphSourceProjection;
use worth_query::facade::{domain, foundation, runtime};

pub struct FinancialSourceProjection {
    record: worth_runtime_bridge::facade::RelationalBridgeRecordIdentityParts,
    secondary_record: Option<worth_runtime_bridge::facade::RelationalBridgeRecordIdentityParts>,
    derive_risk_from_curve: bool,
    unrelated_portfolio_rows: usize,
}

impl FinancialSourceProjection {
    pub const fn curve_risk(
        record: worth_runtime_bridge::facade::RelationalBridgeRecordIdentityParts,
    ) -> Self {
        Self {
            record,
            secondary_record: None,
            derive_risk_from_curve: true,
            unrelated_portfolio_rows: 0,
        }
    }

    pub const fn committed_risk(
        record: worth_runtime_bridge::facade::RelationalBridgeRecordIdentityParts,
    ) -> Self {
        Self {
            record,
            secondary_record: None,
            derive_risk_from_curve: false,
            unrelated_portfolio_rows: 0,
        }
    }

    pub const fn portfolio(
        record: worth_runtime_bridge::facade::RelationalBridgeRecordIdentityParts,
        secondary_record: worth_runtime_bridge::facade::RelationalBridgeRecordIdentityParts,
        unrelated_portfolio_rows: usize,
    ) -> Self {
        Self {
            record,
            secondary_record: Some(secondary_record),
            derive_risk_from_curve: false,
            unrelated_portfolio_rows,
        }
    }
}

impl WorthQueryPrimaryGraphSourceProjection for FinancialSourceProjection {
    fn project_live_target(
        &self,
        graph: &worth_query_execution::facade::integration::WorthQueryPrimaryGraphIntegrationHandle,
        _target: &runtime::WorthQueryLiveArtifactTarget,
    ) -> Result<Vec<foundation::WorthQueryEntity>, foundation::WorthQueryWorkspaceError> {
        let mut rows = project_financial_record(graph, self.record, self.derive_risk_from_curve)?
            .into_iter()
            .collect::<Vec<_>>();
        if let Some(record) = self.secondary_record {
            rows.extend(project_financial_record(
                graph,
                record,
                self.derive_risk_from_curve,
            )?);
        }
        rows.extend(unrelated_portfolio_rows(
            self.record,
            self.unrelated_portfolio_rows,
        ));
        Ok(rows)
    }

    fn project_granular_scope(
        &self,
        graph: &worth_query_execution::facade::integration::WorthQueryPrimaryGraphIntegrationHandle,
        _target: &runtime::WorthQueryLiveArtifactTarget,
        scope: &domain::WorthQueryMaintenanceScope,
        basis: &runtime::WorthQueryGranularSourceReadBasis,
    ) -> Result<Vec<foundation::WorthQueryEntity>, foundation::WorthQueryWorkspaceError> {
        let requested = match scope {
            domain::WorthQueryMaintenanceScope::ExactSourceRecord {
                partition_id,
                local_slot,
                generation,
            } => worth_runtime_bridge::facade::RelationalBridgeRecordIdentityParts::entity(
                *partition_id,
                *local_slot,
                *generation,
            ),
            domain::WorthQueryMaintenanceScope::SourcePartition(partition)
                if partition == "usd-rates" =>
            {
                return self.project_records_at_basis(graph, basis)
            }
            domain::WorthQueryMaintenanceScope::WholeLogicalGraph => {
                return self.project_records_at_basis(graph, basis);
            }
            domain::WorthQueryMaintenanceScope::SourcePartition(_) => return Ok(Vec::new()),
        };
        if requested != self.record && self.secondary_record != Some(requested) {
            return Ok(Vec::new());
        }
        Ok(
            project_financial_record_at_basis(
                graph,
                basis,
                requested,
                self.derive_risk_from_curve,
            )?
            .into_iter()
            .collect(),
        )
    }
}

impl FinancialSourceProjection {
    fn project_records_at_basis(
        &self,
        graph: &worth_query_execution::facade::integration::WorthQueryPrimaryGraphIntegrationHandle,
        basis: &runtime::WorthQueryGranularSourceReadBasis,
    ) -> Result<Vec<foundation::WorthQueryEntity>, foundation::WorthQueryWorkspaceError> {
        std::iter::once(self.record)
            .chain(self.secondary_record)
            .map(|record| {
                project_financial_record_at_basis(graph, basis, record, self.derive_risk_from_curve)
            })
            .collect::<Result<Vec<_>, _>>()
            .map(|rows| rows.into_iter().flatten().collect())
    }
}

fn unrelated_portfolio_rows(
    primary: worth_runtime_bridge::facade::RelationalBridgeRecordIdentityParts,
    count: usize,
) -> Vec<foundation::WorthQueryEntity> {
    (0..count)
        .filter_map(|offset| {
            let record = worth_runtime_bridge::facade::RelationalBridgeRecordIdentityParts::entity(
                primary.partition_id(),
                100_000 + offset as u64,
                1,
            );
            Some(foundation::WorthQueryEntity::from_native_field_values(
                foundation::WorthQueryEntityIdentity::from_bridge_record_projection(record),
                BTreeMap::from([
                    (
                        path("PortfolioFacts", "PortfolioValueField")?,
                        AspectValue::UInt64(1_000 + offset as u64),
                    ),
                    (
                        path("PortfolioFacts", "PortfolioDeskField")?,
                        AspectValue::String("rates".into()),
                    ),
                    (
                        path("PortfolioFacts", "PortfolioRankField")?,
                        AspectValue::UInt64(if offset == 0 {
                            2
                        } else {
                            10_000 + offset as u64
                        }),
                    ),
                ]),
            ))
        })
        .collect()
}

fn project_financial_record(
    graph: &worth_query_execution::facade::integration::WorthQueryPrimaryGraphIntegrationHandle,
    record: worth_runtime_bridge::facade::RelationalBridgeRecordIdentityParts,
    derive_risk_from_curve: bool,
) -> Result<Option<foundation::WorthQueryEntity>, foundation::WorthQueryWorkspaceError> {
    Ok(graph.with_runtime(|runtime| {
        let identity = runtime.main_branch_identity();
        let (_, basis) = runtime.observe_branch(&identity).ok()?;
        let observation = basis.observation();
        project_financial_observation_record(runtime, &observation, record, derive_risk_from_curve)
    })?)
}

fn project_financial_record_at_basis(
    graph: &worth_query_execution::facade::integration::WorthQueryPrimaryGraphIntegrationHandle,
    basis: &runtime::WorthQueryGranularSourceReadBasis,
    record: worth_runtime_bridge::facade::RelationalBridgeRecordIdentityParts,
    derive_risk_from_curve: bool,
) -> Result<Option<foundation::WorthQueryEntity>, foundation::WorthQueryWorkspaceError> {
    graph
        .relational_bridge_source()
        .read_retained_entity_aspect_state(basis.snapshot(), basis.branch(), record)
        .map(|state| {
            state.and_then(|state| {
                project_financial_aspect_state(record, &state, derive_risk_from_curve)
            })
        })
        .map_err(|error| foundation::WorthQueryWorkspaceError::new(error.to_string()))
}

fn project_financial_observation_record(
    runtime: &worth_relational::facade::runtime::RelationalRuntime,
    observation: &worth_relational::facade::branch::RelationalBranchObservation,
    record: worth_runtime_bridge::facade::RelationalBridgeRecordIdentityParts,
    derive_risk_from_curve: bool,
) -> Option<foundation::WorthQueryEntity> {
    let read_view = runtime.read_truth().read_observation(observation).ok()?;
    let entity = worth_relational::facade::identity::EntityId::new(
        worth_relational::facade::identity::PartitionId::new(record.partition_id()),
        record.local_slot(),
        record.generation(),
    );
    let authoritative = read_view
        .entities()
        .iter()
        .find(|candidate| candidate.entity_id == entity)?;
    project_financial_aspect_state(
        record,
        authoritative.authoritative_aspect_state.as_ref()?,
        derive_risk_from_curve,
    )
}

fn project_financial_aspect_state(
    record: worth_runtime_bridge::facade::RelationalBridgeRecordIdentityParts,
    state: &worth_foundational::facade::AuthoritativeRecordAspectState,
    derive_risk_from_curve: bool,
) -> Option<foundation::WorthQueryEntity> {
    let mut fields = BTreeMap::new();
    for (aspect, value) in state.aspects().entries() {
        let ContractValidatedAspectValueView::Struct(value) = value.view() else {
            continue;
        };
        let aspect_key = FieldKey::new(aspect.as_str().to_owned())?;
        fields.extend(value.fields().filter_map(|(field, value)| {
            CanonicalFieldPath::new(vec![aspect_key.clone(), field.clone()])
                .map(|path| (path, value.clone()))
        }));
    }
    if derive_risk_from_curve {
        let curve_path = path("CurveFacts", "CurveZeroRateField")?;
        let AspectValue::UInt64(curve_rate) = fields.get(&curve_path)? else {
            return None;
        };
        fields.insert(
            path("RiskFacts", "RiskValueField")?,
            AspectValue::UInt64(5_100 + curve_rate.saturating_sub(4_250) * 2),
        );
    }
    Some(foundation::WorthQueryEntity::from_native_field_values(
        foundation::WorthQueryEntityIdentity::from_bridge_record_projection(record),
        fields,
    ))
}

fn path(aspect: &str, field: &str) -> Option<CanonicalFieldPath> {
    CanonicalFieldPath::new(vec![FieldKey::new(aspect)?, FieldKey::new(field)?])
}
