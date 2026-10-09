//! Admission on the exact stored scalar before any carrier clone or decoding.
use super::{DecisionReader, HandlerInterruption};
use crate::domain_computation::primary_graph::{
    HandlerExecutionDenial, WorthQueryInvariantEntityIdentity,
};
use worth_foundational::facade::AspectValue as ApplicationValue;
use worth_query_declaration::facade::application_operation::ApplicationMutationBinding;
use worth_query_installation::facade::{
    ApplicationFieldRef, ApplicationFieldUnit, ApplicationReadableScalarValueBinding,
    ApplicationSchema, DeclaredApplicationFieldValue, OperationReads, WritePosture,
};

impl<Schema, Binding> DecisionReader<'_, '_, '_, Schema, Binding>
where
    Schema: ApplicationSchema,
    Binding: ApplicationMutationBinding<Schema>,
{
    /// Admit owner work/storage from the original stored scalar, then use the
    /// same declared binding decoder as [`Self::field`]. This preserves the exact
    /// field dependency, including absence, and never grants raw graph authority.
    /// No scalar/carrier copy occurs before `admit`; bounded projection metadata
    /// and the existing field-read work are still Query-owned costs.
    ///
    /// The callback may inspect the actual carrier's length/capacity and perform
    /// a bounded, metered preflight. It receives a borrowed checkpoint for this
    /// same admitted request so a scan can poll cancellation/deadline while the
    /// reader is borrowed. The callback must admit its own scan, decoder CPU,
    /// carrier copies and peak/retained allocation on its original owner ledger.
    /// Neither the raw loan nor checkpoint may escape. Query does not infer a
    /// domain budget or issue a numerical/current-output proof from admission.
    ///
    /// The outer error is a Query execution failure; request interruption can
    /// be recovered with `error.downcast::<HandlerInterruption>()`. The inner
    /// error retains the callback's concrete denial. As with `field`, lawful
    /// absence or binding decode rejection yields `Ok(Ok(None))`. Admission is
    /// not called for an absent field. Interruption is checked before admission,
    /// before decoding and after completion, including callback denial.
    ///
    /// The borrowed raw value cannot become retained handler state:
    /// ```compile_fail
    /// use worth_foundational::facade::AspectValue;
    /// use worth_query_execution::facade::primary_graph::{
    ///     DecisionReader, WorthQueryInvariantEntityIdentity,
    /// };
    /// use worth_query_declaration::facade::{
    ///     application_operation::ApplicationMutationBinding,
    ///     application_schema::{ApplicationSchema, ApplicationFieldRef,
    ///         ApplicationFieldUnit, ApplicationReadableScalarValueBinding,
    ///         DeclaredApplicationFieldValue, OperationReads, WritePosture},
    /// };
    /// fn retain_raw<'owner, S, B, E, A, F, V, W, Q, U>(
    ///     reader: &mut DecisionReader<'_, '_, '_, S, B>,
    ///     identity: &WorthQueryInvariantEntityIdentity<S, E>,
    ///     field: ApplicationFieldRef<S, E, A, F, V, W, Q, U>,
    ///     retained: &mut Option<&'owner AspectValue>,
    /// ) where S: ApplicationSchema, B: ApplicationMutationBinding<S>,
    ///     F: OperationReads<B::Operation> + DeclaredApplicationFieldValue<Value = V>,
    ///     F::Binding: ApplicationReadableScalarValueBinding,
    ///     W: WritePosture, U: ApplicationFieldUnit,
    /// {
    ///     reader.field_with_predecode_admission(identity, field, |raw, _| {
    ///         *retained = Some(raw); // the loan belongs only to this callback
    ///         Ok::<(), ()>(())
    ///     }).unwrap().unwrap();
    /// }
    /// ```
    #[allow(clippy::type_complexity)]
    pub fn field_with_predecode_admission<
        Entity,
        Aspect,
        Field,
        Value,
        Write,
        Equality,
        Unit,
        Denied,
    >(
        &mut self,
        identity: &WorthQueryInvariantEntityIdentity<Schema, Entity>,
        field: ApplicationFieldRef<Schema, Entity, Aspect, Field, Value, Write, Equality, Unit>,
        admit: impl for<'raw, 'checkpoint> FnOnce(
            &'raw ApplicationValue,
            &'checkpoint dyn Fn() -> Result<(), HandlerInterruption>,
        ) -> Result<(), Denied>,
    ) -> Result<Result<Option<Value>, Denied>, HandlerExecutionDenial>
    where
        Field: OperationReads<Binding::Operation> + DeclaredApplicationFieldValue<Value = Value>,
        Field::Binding: ApplicationReadableScalarValueBinding,
        Write: WritePosture,
        Unit: ApplicationFieldUnit,
    {
        self.checkpoint().map_err(HandlerExecutionDenial::new)?;
        let request = self.execution.request();
        let checkpoint = || {
            request
                .interruption()
                .map_or(Ok(()), |cause| Err(HandlerInterruption::from(cause)))
        };
        let result = self.reader.decision_field_with_predecode_admission(
            identity,
            field,
            admit,
            &checkpoint,
        );
        checkpoint().map_err(HandlerExecutionDenial::new)?;
        result
    }
}
