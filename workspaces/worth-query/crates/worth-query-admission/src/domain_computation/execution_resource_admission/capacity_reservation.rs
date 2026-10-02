use super::{
    WorthQueryAdmittedExecutionResourcePlan, WorthQueryAdmittedWorkflowResourcePlan,
    WorthQueryExecutionCapacityReservation, WorthQueryExecutionResourceSupport,
};

mod admitted_direct;
pub(crate) use admitted_direct::reserve_execution_resource_plan_admitted;

pub struct WorthQueryCapacityReservedExecutionResourcePlan {
    resources: WorthQueryAdmittedExecutionResourcePlan,
    provider_reservations: Vec<Box<dyn WorthQueryExecutionCapacityReservation>>,
    release_identity: String,
}

pub(crate) struct WorthQueryReservedGraphProviderCapacity {
    support_identity: String,
    reservation: Box<dyn WorthQueryExecutionCapacityReservation>,
}

impl WorthQueryReservedGraphProviderCapacity {
    pub(crate) fn release(self) -> WorthQueryExecutionCapacityReleaseReceipt {
        drop(self.reservation);
        WorthQueryExecutionCapacityReleaseReceipt {
            resource_plan_identity: self.support_identity,
            scope: WorthQueryExecutionCapacityReservationScope::GraphWork,
            released_reservation_count: 1,
        }
    }
}

impl WorthQueryCapacityReservedExecutionResourcePlan {
    pub fn resources(&self) -> &WorthQueryAdmittedExecutionResourcePlan {
        &self.resources
    }

    pub fn resources_mut(&mut self) -> &mut WorthQueryAdmittedExecutionResourcePlan {
        &mut self.resources
    }

    pub fn reservation_count(&self) -> usize {
        self.provider_reservations.len()
    }

    pub fn release(self) -> WorthQueryExecutionCapacityReleaseReceipt {
        let released_reservation_count = self.provider_reservations.len();
        drop(self.provider_reservations);
        WorthQueryExecutionCapacityReleaseReceipt {
            resource_plan_identity: self.release_identity,
            scope: WorthQueryExecutionCapacityReservationScope::Direct,
            released_reservation_count,
        }
    }
}

pub struct WorthQueryCapacityReservedWorkflowResourcePlan {
    resources: WorthQueryAdmittedWorkflowResourcePlan,
    provider_reservations: Vec<Box<dyn WorthQueryExecutionCapacityReservation>>,
}

impl WorthQueryCapacityReservedWorkflowResourcePlan {
    fn reserve(resources: WorthQueryAdmittedWorkflowResourcePlan) -> Option<Self> {
        let reservations = reserve_plans(
            std::iter::once(resources.operation())
                .chain(resources.stages().map(|(_, stage)| stage)),
        )?;
        Some(Self {
            resources,
            provider_reservations: reservations,
        })
    }

    pub fn resources(&self) -> &WorthQueryAdmittedWorkflowResourcePlan {
        &self.resources
    }

    pub fn resources_mut(&mut self) -> &mut WorthQueryAdmittedWorkflowResourcePlan {
        &mut self.resources
    }

    pub fn reservation_count(&self) -> usize {
        self.provider_reservations.len()
    }

    pub fn release(self) -> WorthQueryExecutionCapacityReleaseReceipt {
        release_reservations(
            self.resources.identity(),
            self.provider_reservations,
            WorthQueryExecutionCapacityReservationScope::Workflow,
        )
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum WorthQueryExecutionCapacityReservationScope {
    Direct,
    Workflow,
    GraphWork,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct WorthQueryExecutionCapacityReleaseReceipt {
    resource_plan_identity: String,
    scope: WorthQueryExecutionCapacityReservationScope,
    released_reservation_count: usize,
}

impl WorthQueryExecutionCapacityReleaseReceipt {
    pub fn resource_plan_identity(&self) -> &str {
        &self.resource_plan_identity
    }

    pub fn scope(&self) -> WorthQueryExecutionCapacityReservationScope {
        self.scope
    }

    pub fn released_reservation_count(&self) -> usize {
        self.released_reservation_count
    }
}

pub fn reserve_execution_resource_plan(
    resources: WorthQueryAdmittedExecutionResourcePlan,
) -> Option<WorthQueryCapacityReservedExecutionResourcePlan> {
    match reserve_execution_resource_plan_admitted(resources, &mut |_, _| {
        Ok::<(), std::convert::Infallible>(())
    }) {
        Ok(reserved) => reserved,
        Err(WorthQueryCapacityReservationAdmissionStop::Admission(never)) => match never {},
        Err(WorthQueryCapacityReservationAdmissionStop::AccountingOverflow) => None,
    }
}

pub fn reserve_workflow_resource_plan(
    mut resources: WorthQueryAdmittedWorkflowResourcePlan,
) -> Option<WorthQueryCapacityReservedWorkflowResourcePlan> {
    resources.record_capacity_reservation_check();
    let mut reserved = WorthQueryCapacityReservedWorkflowResourcePlan::reserve(resources)?;
    reserved.resources_mut().record_capacity_reservation();
    Some(reserved)
}

pub(crate) fn reserve_graph_provider_capacity(
    support: &WorthQueryExecutionResourceSupport,
) -> Option<WorthQueryReservedGraphProviderCapacity> {
    Some(WorthQueryReservedGraphProviderCapacity {
        support_identity: support.identity().to_owned(),
        reservation: support.capacity().try_reserve()?,
    })
}

pub(crate) fn reserve_graph_provider_capacity_admitted<Stop>(
    support: &WorthQueryExecutionResourceSupport,
    admit: &mut impl FnMut(u64, u64) -> Result<(), Stop>,
) -> Result<
    Option<WorthQueryReservedGraphProviderCapacity>,
    WorthQueryCapacityReservationAdmissionStop<Stop>,
> {
    // The capacity implementation owns its atomic/Arc/Box quote. The support
    // owner knows the exact copied identity that the reservation retains.
    admit(2, 0).map_err(WorthQueryCapacityReservationAdmissionStop::Admission)?;
    let identity_bytes = u64::try_from(support.identity().len())
        .map_err(|_| WorthQueryCapacityReservationAdmissionStop::AccountingOverflow)?;
    let (reservation_work, reservation_bytes) = support
        .capacity()
        .reservation_preflight_cost()
        .ok_or(WorthQueryCapacityReservationAdmissionStop::AccountingOverflow)?;
    admit(identity_bytes, identity_bytes)
        .map_err(WorthQueryCapacityReservationAdmissionStop::Admission)?;
    admit(reservation_work, reservation_bytes)
        .map_err(WorthQueryCapacityReservationAdmissionStop::Admission)?;
    Ok(reserve_graph_provider_capacity(support))
}

#[derive(Debug)]
pub(crate) enum WorthQueryCapacityReservationAdmissionStop<Stop> {
    Admission(Stop),
    AccountingOverflow,
}

fn release_reservations(
    resource_plan_identity: &str,
    reservations: Vec<Box<dyn WorthQueryExecutionCapacityReservation>>,
    scope: WorthQueryExecutionCapacityReservationScope,
) -> WorthQueryExecutionCapacityReleaseReceipt {
    let released_reservation_count = reservations.len();
    drop(reservations);
    WorthQueryExecutionCapacityReleaseReceipt {
        resource_plan_identity: resource_plan_identity.to_owned(),
        scope,
        released_reservation_count,
    }
}

fn reserve_plans<'a>(
    plans: impl IntoIterator<Item = &'a WorthQueryAdmittedExecutionResourcePlan>,
) -> Option<Vec<Box<dyn WorthQueryExecutionCapacityReservation>>> {
    let supports = exact_capacity_subjects(plans)?;
    let mut reservations = Vec::with_capacity(supports.len());
    for support in supports {
        reservations.push(support.capacity().try_reserve()?);
    }
    Some(reservations)
}

fn exact_capacity_subjects<'a>(
    plans: impl IntoIterator<Item = &'a WorthQueryAdmittedExecutionResourcePlan>,
) -> Option<Vec<&'a WorthQueryExecutionResourceSupport>> {
    let mut requested = Vec::new();
    for plan in plans {
        for support in plan.support_snapshot().all_supports() {
            retain_exact_support(&mut requested, support)?;
        }
    }
    Some(requested)
}

fn retain_exact_support<'a>(
    requested: &mut Vec<&'a WorthQueryExecutionResourceSupport>,
    support: &'a WorthQueryExecutionResourceSupport,
) -> Option<()> {
    if let Some(existing) = requested.iter().find(|existing| {
        existing.capacity_subject_identity() == support.capacity_subject_identity()
    }) {
        return existing.has_same_capacity_authority(support).then_some(());
    }
    requested.push(support);
    Some(())
}
