use super::*;
impl InspectionAccess<'_> {
    pub(super) fn connectivity_components(
        &self,
        request: &ConnectivityInspectionRequest,
        _version_id: VersionId,
        entities: &[EntityId],
        adjacency: &BTreeMap<EntityId, BTreeSet<EntityId>>,
        mut work: ConnectivityWork,
        context: &mut worth_execution::MapKernelContext<'_, '_>,
    ) -> Result<
        Result<
            (Vec<ConnectivityComponentSummary>, ConnectivityWork),
            (ConnectivityWork, InspectionDegradation),
        >,
        crate::execution::RelationalExecutionDenialCause,
    > {
        let mut visited = BTreeSet::new();
        let mut components = Vec::new();
        for entity in entities {
            if !visited.insert(*entity) {
                continue;
            }
            context
                .checkpoint(1)
                .map_err(crate::execution::denial::kernel_stop)?;
            work.record_component();
            if work.work_units > request.budget.max_work_units {
                return Ok(Err((work, InspectionDegradation::WorkBudgetExceeded)));
            }
            if work.components_evaluated > request.budget.max_components {
                return Ok(Err((work, InspectionDegradation::ComponentBudgetExceeded)));
            }
            let mut queue = VecDeque::from([*entity]);
            let mut members = vec![*entity];
            while let Some(current) = queue.pop_front() {
                context
                    .checkpoint(1)
                    .map_err(crate::execution::denial::kernel_stop)?;
                work.record_frontier_expansion();
                if work.work_units > request.budget.max_work_units {
                    return Ok(Err((work, InspectionDegradation::WorkBudgetExceeded)));
                }
                if queue.len() as u64 > request.budget.max_frontier {
                    return Ok(Err((work, InspectionDegradation::FrontierBudgetExceeded)));
                }
                for neighbor in adjacency.get(&current).into_iter().flatten() {
                    if visited.insert(*neighbor) {
                        members.push(*neighbor);
                        queue.push_back(*neighbor);
                    }
                }
            }
            members.sort();
            components.push(ConnectivityComponentSummary {
                member_count: members.len() as u64,
                members: request.include_members.then_some(members),
            });
        }
        Ok(Ok((components, work)))
    }
}
