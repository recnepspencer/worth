//! C operation permission precedes selecting the genuine predecessor's old truth.
use super::*;
pub(super) use crate::domain_computation::primary_graph::application_contribution::producer::predecessor_reads::WorthQueryPredecessorReadDenial as Denial;
use worth_query_declaration::facade::application_schema::{ApplicationOperationMarkerIdentity, ApplicationOperationRef};
use worth_relational::facade::{runtime::RelationalRuntime, mvcc::{RelationalPredecessorInput, RelationalRequiredOutputPublication}};

impl<S: ApplicationSchema> SourceQueryReaderAccess<'_, S> {
    pub(in crate::domain_computation::primary_graph) fn predecessor_control(
        &self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        native: &RelationalRuntime,
    ) -> Result<(), Denial> {
        if let Some(cause) = self.predecessor_refused.borrow().as_ref() {
            return Err(cause.clone());
        }
        self.confirm_control(
            execution,
            &mut ApplicationQueryNativeAccess::Borrowed(native),
            (),
        )
        .map_err(|cause| Denial::Control(Box::new(self.record_control_refusal(cause))))
    }
    pub(in crate::domain_computation::primary_graph) fn record_predecessor_failure(
        &self,
        cause: &Denial,
    ) {
        // First/cold/restoration unavailability is an explicit data posture.
        // A handler may choose full computation, but cannot swallow admission loss.
        let data_posture = match cause {
            Denial::PredecessorBasisUnavailable => true,
            Denial::Native(
                worth_relational::facade::mvcc::RelationalPredecessorBasisDenial::PredecessorBasisUnavailableForOrigin(origin),
            ) => origin_is_data_posture(*origin),
            _ => false,
        };
        if !data_posture {
            self.predecessor_refused
                .borrow_mut()
                .get_or_insert_with(|| cause.clone());
        }
    }
    pub(in crate::domain_computation::primary_graph) fn prepare_predecessor<O, I>(
        &self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        native: &RelationalRuntime,
        operation: ApplicationOperationRef<S, O, I>,
        scope: worth_relational::facade::identity::EntityId,
        partition: [u8; 32],
    ) -> Result<(
        RelationalRequiredOutputPublication,
        RelationalPredecessorInput,
        Arc<worth_query_installation::facade::WorthQueryCompiledApplicationOperationContracts>,
        Arc<crate::domain_computation::primary_graph::application_attempt::SourceReadAuthorization>,
    ), Denial>
    where O: ApplicationOperationMarkerIdentity<S> + 'static,
        O::InputBinding: ApplicationStructuredValueBinding<Value = I>, I: 'static,
{
        self.predecessor_control(execution, native)?;
        let installed = self
            .application
            .installed_schema()
            .installed_operation(operation)
            .map_err(Denial::Installation)?;
        self.application
            .runtime
            .installed_packages()
            .validate_application_operation(&installed)
            .map_err(Denial::Installation)?;
        let current = self
            .history
            .as_ref()
            .map_or(self.shared, |control| control.current)
            .selected();
        let authorization = self
            .read_authorization
            .as_ref()
            .ok_or(Denial::UndeclaredPredecessorRead)?
            .observe(
                execution,
                native,
                current.application_basis().snapshot_handle().clone(),
                installed.installed_view(),
            )
            .map_err(Denial::Authorization)?;
        let access = self
            .predecessor
            .as_ref()
            .ok_or(Denial::UndeclaredPredecessorRead)?;
        access.require::<S, O>()?;
        let (prior, basis) = access.select(
            execution,
            self.application,
            native,
            self.shared.selected().product().relational_basis(),
            scope,
            partition,
        )?;
        let authorization = crate::domain_computation::primary_graph::application_attempt::SourceReadAuthorization::retain_operation(
            execution, installed.installed_view(), self.read_authorization.as_ref().expect("admitted read"),
            authorization, &self.application.runtime.authorization,
        ).map_err(|stop| Denial::Control(Box::new(SourceQueryObservationStop::Read(WorthQueryAdmittedOneShotStop::Admission(stop)))))?;
        let contracts = installed.retain_compiled_contracts_for_admission();
        Ok((prior, basis, contracts, authorization))
    }
    pub(in crate::domain_computation::primary_graph) fn record_predecessor_read(
        &self,
        read: crate::domain_computation::primary_graph::application_contribution::producer::predecessor_reads::WorthQueryHistoricalProducerRead,
    ) {
        self.predecessor
            .as_ref()
            .expect("selection requires declared predecessor reads")
            .state
            .record(read);
    }
}

/// Each origin Native defines is decided here, with no wildcard: a new origin
/// fails to compile until someone decides it (CP2b slice 4, review condition
/// 7). HistoricalSelection is courted (`owned_outputs/predecessor/origin_kind.rs`).
/// AuthoredRestoration as a data posture is not courted in Query: no public
/// fixture republishes a predecessor-reading producer's output by authored
/// restoration (the fixture's predecessor family is Model-rooted; Model retire
/// deletes the document root). Courted where CAD reaches it: public #138.
fn origin_is_data_posture(
    origin: worth_relational::facade::mvcc::RelationalPredecessorBasisOrigin,
) -> bool {
    use worth_relational::facade::mvcc::RelationalPredecessorBasisOrigin as Origin;
    match origin {
        Origin::HistoricalSelection | Origin::AuthoredRestoration => true,
    }
}
