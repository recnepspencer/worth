use std::sync::Arc;

use crate::branch::{
    ProductBranchObservation, ProductBranchReferenceCell, ProductBranchReferenceLoss,
    ProductBranchReferenceSnapshot,
};
use crate::history::{CompositeRuntimeWorldCommit, PreparedPublicationRecord};
use crate::publication::{
    CompositeLateCancellationPosture, CompositeOwnerExecutionResults,
    CompositePublicationCostCounters, PerformedCompositePublication,
};

use super::{ActiveAttemptCustody, ActiveHistoryCustody, ActivePinCustody};

pub(crate) enum AttemptProductMovementFailure {
    Observation(crate::retention::RetentionObligationDenial),
    Reference(ProductBranchReferenceLoss),
}

impl ActiveAttemptCustody {
    /// Full component evidence is prepared before the cell lock. The same
    /// owner-held resource lease spans materialization, CAS, and caller unwind.
    pub(crate) fn attempt_movement(
        &mut self,
        expected: &ProductBranchObservation,
        commit: &Arc<CompositeRuntimeWorldCommit>,
        results: &CompositeOwnerExecutionResults,
        counters: &mut CompositePublicationCostCounters,
        late: CompositeLateCancellationPosture,
        cell: &ProductBranchReferenceCell,
        cutoff: Option<crate::publication::ProductMovementCutoff>,
    ) -> Result<PerformedCompositePublication, AttemptProductMovementFailure> {
        let publication = self
            .record
            .publication
            .as_ref()
            .cloned()
            .expect("ordinary custody reserves a publication envelope");
        let snapshot = ProductBranchReferenceSnapshot::owner_issued(
            expected.owner_identity(),
            expected.branch_identity().clone(),
            expected.lifecycle_incarnation(),
            expected
                .reference_generation()
                .advance()
                .expect("generation was admitted before effects"),
            Arc::clone(commit),
        )
        .expect("the successor has the expected owner and lineage");
        self.prepare_successor_observation(&snapshot)
            .map_err(AttemptProductMovementFailure::Observation)?;
        let mut lease = self.lease_resources();
        counters.record_product_cell_touch();
        counters.record_cas_attempt();
        let mut performed_counters = *counters;
        performed_counters.record_history_slot_installed();
        let record: PreparedPublicationRecord = publication
            .prepare(commit, results, late, performed_counters)
            .with_cutoff(cutoff);
        if let Err(loss) = cell.publish_recorded(
            expected,
            &mut lease,
            |lease| {
                lease.install_successor(commit, snapshot);
                counters.record_history_slot_installed();
                &mut lease.resources_mut().product_head
            },
            record,
        ) {
            if loss.cutoff_denial().is_none() {
                counters.record_cas_loss();
            }
            lease.resources_mut().product_comparison_costs = Some(*counters);
            return Err(AttemptProductMovementFailure::Reference(loss));
        }
        let resources = lease.resources_mut();
        let delivery = resources
            .delivery
            .take()
            .expect("successful custody reserved its exclusive delivery");
        Ok(PerformedCompositePublication::owner_issued(delivery))
    }

    fn prepare_successor_observation(
        &mut self,
        snapshot: &ProductBranchReferenceSnapshot,
    ) -> Result<(), crate::retention::RetentionObligationDenial> {
        let mut lease = self.lease_resources();
        let resources = lease.resources_mut();
        let Some(capacity) = resources.successor_observation_capacity.take() else {
            assert!(resources.successor_observation_pins.is_none());
            return Ok(());
        };
        let pins = resources
            .successor_observation_pins
            .as_mut()
            .expect("successor observation capacity reserves its component pair");
        let components = pins.try_bind_observation(snapshot.commit(), capacity)?;
        resources.successor_observation_pins = None;
        resources.prepared_successor_observation = Some(
            ProductBranchObservation::prepare_successor(snapshot.clone(), components),
        );
        Ok(())
    }
}

impl super::ActiveAttemptResourceLease<'_> {
    fn install_successor(
        &mut self,
        commit: &Arc<CompositeRuntimeWorldCommit>,
        snapshot: ProductBranchReferenceSnapshot,
    ) {
        let resources = self.resources_mut();
        let ActivePinCustody::Bound(pins) = &resources.pins else {
            panic!("a ready publication holds its exact bound pins")
        };
        assert!(
            pins.matches_basis(commit.basis()),
            "installed claims bind the admitted successor before history promotion"
        );
        let ActiveHistoryCustody::Reserved(capacity) = &mut resources.history_custody else {
            panic!("a ready publication holds its uninstalled slot")
        };
        let (history, delivery) = capacity
            .try_install_publication(
                Arc::clone(commit),
                resources
                    .history_pins
                    .take()
                    .expect("history pins bound before cell admission"),
                resources.prepared_successor_observation.take(),
            )
            .expect("reserved publication history installs with both protections");
        resources.history_custody = ActiveHistoryCustody::Installed(history);
        resources.delivery = Some(delivery);
        self.assemble_head(snapshot);
    }
}
