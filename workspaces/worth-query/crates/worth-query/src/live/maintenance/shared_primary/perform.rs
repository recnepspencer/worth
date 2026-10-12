//! Execute one prepared shared primary projection maintenance.
use super::*;

pub fn perform_prepared_shared_primary_runtime_granular_maintenance<
    D: 'static,
    O: 'static,
    F: 'static,
    L: BasisOperationLane,
>(
    execution: &worth_execution::ExecutionRequest<'_, '_>,
    prepared: WorthQueryPreparedSharedPrimaryGranularMaintenance,
    consumers: &[&WorthQuerySharedLiveProjectionLease<D, O, F, L>],
    workspace: &mut crate::runtime::WorthQueryWorkspace,
    binding: &super::super::WorthQueryPrimaryRuntimeInvalidationBinding,
) -> Result<
    WorthQuerySharedPrimaryGranularMaintenanceOutcome,
    WorthQuerySharedPrimaryGranularMaintenanceDenial,
> {
    let Some(first) = consumers.first().copied() else {
        return Err(WorthQuerySharedPrimaryGranularMaintenanceDenial::EmptyConsumerSet);
    };
    if !binding.readmits_workspace(workspace) {
        return Err(WorthQuerySharedPrimaryGranularMaintenanceDenial::ForeignPrimaryRuntime);
    }
    let WorthQueryPreparedSharedPrimaryGranularMaintenance {
        owner,
        selected_consumers,
        plan,
        impacts,
        admission_counters,
        coverage,
    } = prepared;
    let current_consumers = admitted_consumer_set(consumers, owner, workspace)?;
    if !current_consumers.keys().all(|lease| {
        selected_consumers
            .get(lease)
            .is_some_and(|selected| selected == &current_consumers[lease])
    }) {
        return Err(WorthQuerySharedPrimaryGranularMaintenanceDenial::ConsumerSetMismatch);
    }
    let refresh = workspace
        .refresh_shared_primary_owner(
            execution,
            first.workspace_capability(),
            first.readmission(),
            first.snapshot(),
            plan.scope(),
            plan.source_read_basis()
                .ok_or(WorthQuerySharedPrimaryGranularMaintenanceDenial::MixedMaintenancePosture)?,
        )
        .map_err(|stop| match stop {
            crate::runtime::WorthQuerySharedPrimaryOwnerRefreshStop::Runtime(error) => {
                WorthQuerySharedPrimaryGranularMaintenanceDenial::Runtime(error)
            }
            crate::runtime::WorthQuerySharedPrimaryOwnerRefreshStop::Refresh(error) => {
                WorthQuerySharedPrimaryGranularMaintenanceDenial::Execution(error)
            }
        })?;
    let execution_identity = refresh.authority().receipt().receipt_digest().to_owned();
    let admitted_impact_count = impacts.len();
    let impact_observations = impacts
        .iter()
        .map(crate::domain_installation::WorthQueryAdmittedInvalidationImpact::observation)
        .collect::<Vec<_>>();
    let maintenance_owner = format!("shared-primary:{}", owner.slot());
    let full_scope = coverage == WorthQueryGranularInvalidationCoverage::RefreshAll;
    let (projection, maintained_impacts) = prepare_covered_projection_maintenance(
        workspace,
        super::super::WorthQueryProjectionMaintenanceRequest {
            owner: &maintenance_owner,
            plan: &plan,
            impacts: &impacts,
            current: first.snapshot(),
            refresh: &refresh,
        },
        full_scope,
    );
    let derived = super::super::derive_performed_maintenance_effect(
        &plan,
        maintained_impacts,
        first.snapshot(),
        &refresh,
        None,
        projection,
    )
    .map_err(WorthQuerySharedPrimaryGranularMaintenanceDenial::Maintenance)?;
    let Some(derived) = derived else {
        // Only a full-scope refresh may find nothing changed.
        use super::super::WorthQueryMaintenanceDenial::PerformedEffectUnavailable as Unavailable;
        let denied = WorthQuerySharedPrimaryGranularMaintenanceDenial::Maintenance(Unavailable);
        let unchanged = WorthQuerySharedPrimaryGranularMaintenanceOutcome::NoRelevantChange;
        return if full_scope {
            Ok(unchanged)
        } else {
            Err(denied)
        };
    };
    debug_assert!(derived.collection_commit.is_none());
    let effect = derived.effect;
    let mut publications = Vec::new();
    for consumer in consumers {
        if !workspace
            .readmits_shared_primary_lease(consumer.workspace_capability(), consumer.readmission())
        {
            continue;
        }
        let snapshot = consumer.snapshot();
        let lease = consumer.lease_identity();
        let Some(consumer_delivery_authority) =
            current_shared_consumer_delivery_authority(*consumer, workspace)
        else {
            continue;
        };
        if selected_consumers.get(&lease) != Some(&consumer_delivery_authority) {
            continue;
        }
        publications.push(WorthQueryPublishedSharedPrimaryInvalidation {
            owner,
            lease,
            strategies: plan.strategies().to_vec(),
            scope: plan.scope().clone(),
            roles: plan.roles().to_vec(),
            result_state: snapshot.result_state(),
            consumer_binding_identity: snapshot.consumer_contract().binding_identity().to_owned(),
            consumer_operation_identity: snapshot
                .consumer_contract()
                .canonical_operation_identity()
                .to_owned(),
            delivery_identity: crate::identity::hash_parts(&[
                "worth_query_shared_primary_invalidation_v1".into(),
                format!("owner:{}", owner.slot()),
                format!("lease:{}", lease.slot()),
                format!("execution:{execution_identity}"),
                format!(
                    "consumer-authority:{}",
                    consumer_delivery_authority.authority_identity()
                ),
            ]),
            consumer_delivery_authority,
            effect: std::sync::Arc::clone(&effect),
        });
    }
    let denied_consumer_count = selected_consumers.len() - current_consumers.len();
    let maintenance_counters = super::super::WorthQueryGranularMaintenanceCounters::shared(
        &effect,
        admitted_impact_count,
        publications.iter().map(|publication| {
            publication
                .consumer_delivery_authority
                .backpressure_policy()
        }),
        denied_consumer_count,
    );
    workspace.apply_projection_maintenance(&maintenance_owner, derived.projection_commit);
    Ok(
        WorthQuerySharedPrimaryGranularMaintenanceOutcome::Performed(
            WorthQuerySharedPrimaryGranularMaintenancePerformed {
                _refresh: refresh,
                publications,
                denied_consumer_count,
                admission_counters,
                maintenance_counters,
                impact_observations,
            },
        ),
    )
}
