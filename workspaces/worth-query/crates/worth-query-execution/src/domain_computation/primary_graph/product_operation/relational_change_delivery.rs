use worth_query_installation::facade::ApplicationSchema;

use super::WorthQuerySelectedProductOperation;
use crate::domain_computation::execution_runtime::product_world::{
    WorthQueryPerformedRelationalProductChange,
    WorthQueryPerformedRelationalProductChangeDeliveryDenial,
    WorthQueryPerformedRelationalProductChangeDeliveryDenialKind,
    WorthQueryPerformedRelationalProductChangeDeliveryOutcome,
};
use crate::domain_computation::primary_graph::conditional_operation::WorthQueryConditionalClockHandle;

impl<'runtime, Schema: ApplicationSchema> WorthQuerySelectedProductOperation<'runtime, Schema> {
    /// Delivers the unique Relational patch retained by a fresh application
    /// publication to one admitted conditional dependency.
    pub fn deliver_relational_change_to_conditional<Node, Clock>(
        &self,
        handle: &WorthQueryConditionalClockHandle<Schema, Node, Clock>,
        dependency_ordinal: usize,
        change: WorthQueryPerformedRelationalProductChange,
    ) -> Result<
        WorthQueryPerformedRelationalProductChangeDeliveryOutcome,
        WorthQueryPerformedRelationalProductChangeDeliveryDenial,
    > {
        use WorthQueryPerformedRelationalProductChangeDeliveryDenialKind as Kind;

        if change.product_branch_identity() != self.product().branch_identity()
            || change.product_commit() != self.product().selected_commit()
        {
            return Err(
                WorthQueryPerformedRelationalProductChangeDeliveryDenial::new(
                    Kind::ForeignProductOccurrence,
                    "performed product change does not belong to the selected product occurrence",
                    change,
                ),
            );
        }
        let truth = match crate::domain_computation::primary_graph::conditional_operation::WorthQueryConditionalTruthBasis::retain_selected(self) {
            Ok(truth) => truth,
            Err(denial) => {
                return Err(WorthQueryPerformedRelationalProductChangeDeliveryDenial::new(
                    Kind::ProductAdmission,
                    format!("selected product truth retention was denied: {denial:?}"),
                    change,
                ));
            }
        };
        let operation = self
            .application()
            .conditional_operations
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .admit_clock(handle.binding_identity(), handle.lease());
        let Some(operation) = operation else {
            return Err(
                WorthQueryPerformedRelationalProductChangeDeliveryDenial::new(
                    Kind::ForeignConditionalOperation,
                    "conditional handle does not belong to the selected application runtime",
                    change,
                ),
            );
        };
        let bridge_root = self.application().bridge.conditional_operations();
        let bridge = bridge_root
            .read()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        let mut installed = operation.lock_operation();
        if let Err(denial) = installed.select_product_binding(&bridge, self.application(), &truth) {
            return Err(
                WorthQueryPerformedRelationalProductChangeDeliveryDenial::new(
                    Kind::ConditionalProductAdmission,
                    format!(
                        "selected conditional product admission was denied: {}",
                        denial.subject()
                    ),
                    change,
                ),
            );
        }
        let lowering = installed.selected_lowering();
        drop(bridge);
        let outcome = self
            .application()
            .granular_invalidation_installation()
            .retain_product_shared_root()
            .deliver_performed_relational_change(&lowering, dependency_ordinal, change)?;
        if let WorthQueryPerformedRelationalProductChangeDeliveryOutcome::Success(receipt) =
            &outcome
        {
            installed.retain_direct_delivery(receipt);
        }
        let routes_changed = operation.publish_routes(installed.as_ref());
        drop(installed);
        if routes_changed {
            let registry = self
                .application()
                .conditional_operations
                .lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner)
                .snapshot();
            registry.synchronize_commit_routes(self.application());
        }
        Ok(outcome)
    }
}
