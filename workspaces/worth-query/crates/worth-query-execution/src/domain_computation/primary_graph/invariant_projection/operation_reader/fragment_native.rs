//! Installed fragment row reads and resolution share native snapshot and decision owners.
use super::super::super::fragment_view::{ErasedObserved, FragmentDenial};
use super::super::native_read::NativeRead;
use super::decision_plan::native_decision::NativeDecision;
use super::decision_reads::DecisionReadOutcome;
use super::WorthQueryApplicationOperationInvariantProjectionReader as Reader;
use crate::domain_computation::primary_graph::application_attempt::{
    observe_aspect_presence, WorthQueryApplicationFactKey,
};
use crate::domain_computation::primary_graph::{
    HandlerExecutionDenial, WorthQueryApplicationAttemptDenial,
    WorthQueryApplicationAttemptDenialKind,
};
use std::collections::BTreeMap;
use worth_foundational::facade::AspectValue;
use worth_query_declaration::facade::application_schema::fragment::AspectDefinition;
use worth_query_declaration::facade::application_schema::{
    ApplicationFieldBindingLocus, ApplicationOperationDecisionReadTarget,
};
use worth_query_installation::facade::ApplicationSchema;
pub(super) struct NativeFragmentRead<'a, 'r> {
    native: NativeRead<'a, 'r>,
    decision: NativeDecision<'a>,
    resolution:
        &'r crate::domain_computation::primary_graph::WorthQueryInstalledEntityResolutionContext,
}
impl<'r, S: ApplicationSchema, O> Reader<'_, 'r, S, O> {
    pub(super) fn native_fragment(&mut self) -> NativeFragmentRead<'_, 'r> {
        let resolution = self.reader.entity_resolution;
        let native = self.reader.native_read();
        let decision = NativeDecision::new(
            native.layout,
            native.bindings,
            native.authority,
            self.admitted_contracts.as_deref(),
            self.decision_facts,
        );
        NativeFragmentRead {
            native,
            decision,
            resolution,
        }
    }
}
impl NativeFragmentRead<'_, '_> {
    pub(super) fn resolve(
        mut self,
        field: &ApplicationFieldBindingLocus,
        value: AspectValue,
    ) -> Result<ErasedObserved, FragmentDenial> {
        self.decision
            .admit_target(&ApplicationOperationDecisionReadTarget::Field {
                entity: field.entity().to_owned(),
                aspect: field.aspect().to_owned(),
                field: field.field().to_owned(),
            })
            .map_err(HandlerExecutionDenial::new)?;
        let identity = self
            .native
            .entity_resolution(self.resolution)
            .resolve_encoded(field, value)
            .map_err(FragmentDenial::Resolution)?;
        self.decision
            .tracked_field(identity.native_identity(), field)
            .map_err(HandlerExecutionDenial::new)?;
        Ok(ErasedObserved { identity })
    }
    pub(super) fn record(
        &mut self,
        target: &ErasedObserved,
        aspect: &AspectDefinition,
    ) -> Result<Option<BTreeMap<String, AspectValue>>, FragmentDenial> {
        let identity = target.identity.native_identity();
        let first = aspect
            .fields()
            .first()
            .ok_or_else(|| missing(aspect.path()))?;
        let locator = self
            .decision
            .tracked_field(identity, first.recipe().locus())
            .map_err(HandlerExecutionDenial::new)?;
        let key = WorthQueryApplicationFactKey::AspectPresence {
            entity: aspect.entity().to_owned(),
            entity_id: identity.entity_id,
            locator: locator.clone(),
        };
        self.decision
            .record_outcome(key.clone(), DecisionReadOutcome::Observed);
        if !self.native.budget.consume(1) {
            return Err(FragmentDenial::Read(HandlerExecutionDenial::new(WorthQueryApplicationAttemptDenial::from(worth_relational::facade::transactions::RelationalExecutionDenialCause::WorkExhausted))));
        }
        self.native.scope.record(identity.entity_id);
        let present = observe_aspect_presence(
            self.native.runtime,
            self.native.execution,
            self.native.snapshot,
            identity.entity_id,
            identity.kind,
            locator.aspect().aspect_key(),
        );
        let present = match present {
            Ok(Some(present)) => present,
            Ok(None) => {
                self.decision
                    .record_outcome(key, DecisionReadOutcome::Failed);
                return Err(missing(aspect.path()));
            }
            Err(cause) => {
                self.native.stopped.get_or_insert(cause);
                self.decision
                    .record_outcome(key, DecisionReadOutcome::Failed);
                return Err(FragmentDenial::Read(HandlerExecutionDenial::new(
                    WorthQueryApplicationAttemptDenial::from(cause),
                )));
            }
        };
        let mut values = BTreeMap::new();
        for (index, field) in aspect.fields().iter().enumerate() {
            let locus = field.recipe().locus();
            let locator = if index == 0 {
                locator.clone()
            } else {
                self.decision
                    .tracked_field(identity, locus)
                    .map_err(HandlerExecutionDenial::new)?
            };
            let path = format!("{}.{}", aspect.path(), locus.field());
            let decoder = self.native.decoder;
            let mut raw = None;
            self.native
                .observe_field(identity, &locator, &mut |observed| {
                    raw = observed
                        .map(|raw| {
                            let bytes = raw
                                .owned_allocation_capacity_bytes()
                                .checked_add(path.len())
                                .and_then(|bytes| {
                                    bytes.checked_add(
                                        std::mem::size_of::<(
                                            String,
                                            worth_foundational::facade::AspectValue,
                                        )>() + 8 * std::mem::size_of::<usize>(),
                                    )
                                })
                                .and_then(|bytes| u64::try_from(bytes).ok())
                                .ok_or_else(|| {
                                    HandlerExecutionDenial::new(
                                        WorthQueryApplicationAttemptDenial::new(
                                            WorthQueryApplicationAttemptDenialKind::ValueDecode(
                                                decoder.accounting_overflow(),
                                            ),
                                            &path,
                                        ),
                                    )
                                })?;
                            decoder.admit_write_quantities(1, bytes).map_err(|cause| {
                                HandlerExecutionDenial::new(
                                    WorthQueryApplicationAttemptDenial::new(
                                        WorthQueryApplicationAttemptDenialKind::ValueDecode(cause),
                                        &path,
                                    ),
                                )
                            })?;
                            Ok(raw.clone())
                        })
                        .transpose()?;
                    Ok(())
                })?;
            if let Some(raw) = raw.filter(|_| !field.internal_presence()) {
                values.insert(path, raw);
            }
        }
        Ok(present.then_some(values))
    }
}
fn missing(path: &str) -> FragmentDenial {
    FragmentDenial::Read(HandlerExecutionDenial::new(
        WorthQueryApplicationAttemptDenial::new(
            WorthQueryApplicationAttemptDenialKind::MissingAuthoritativeFact,
            path,
        ),
    ))
}

pub(super) fn with_decode_admission(
    session: &crate::domain_computation::primary_graph::value_decode::ValueDecodeSession<'_>,
    path: &str,
    decode: &mut dyn FnMut(
        &mut worth_query_declaration::facade::application_schema::ValueDecodeAdmission<'_>,
    ) -> Result<(), FragmentDenial>,
) -> Result<(), FragmentDenial> {
    session
        .with_record_admission(decode)
        .map_err(|cause| FragmentDenial::DecodeControl {
            path: path.to_owned(),
            cause,
        })?
}
