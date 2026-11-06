//! Fragment reads retain native snapshot reads, decision facts and original controls.

use super::super::super::fragment_view::{ErasedObserved, FragmentDenial, ReadPort, Reading, View};
use super::super::super::{
    HandlerExecutionDenial, WorthQueryApplicationAttemptDenial,
    WorthQueryApplicationAttemptDenialKind,
};
use super::decision_reads::DecisionReadOutcome;
use super::WorthQueryApplicationOperationInvariantProjectionReader as Reader;
use crate::domain_computation::primary_graph::application_attempt::{
    observe_aspect_presence, WorthQueryApplicationFactKey,
};
use std::collections::BTreeMap;
use worth_query_declaration::facade::application_schema::fragment::{AspectDefinition, Fragment};
use worth_query_installation::facade::ApplicationSchema;

impl<S: ApplicationSchema, O> Reader<'_, '_, S, O> {
    pub fn fragment<F: Fragment>(
        &mut self,
        marker: F,
    ) -> Result<View<'_, F, Reading<'_>>, FragmentDenial> {
        let catalog = self.reader.value_bindings;
        let definition = catalog
            .fragment(marker)
            .ok_or(FragmentDenial::NotInstalled { fragment: F::NAME })?;
        Ok(View::reading(definition, self))
    }
}

impl<S: ApplicationSchema, O> ReadPort for Reader<'_, '_, S, O> {
    fn with_decode_admission(
        &mut self,
        path: &str,
        decode: &mut dyn FnMut(
            &mut worth_query_declaration::facade::application_schema::ValueDecodeAdmission<'_>,
        ) -> Result<(), FragmentDenial>,
    ) -> Result<(), FragmentDenial> {
        self.reader
            .value_decode
            .with_record_admission(decode)
            .map_err(|cause| FragmentDenial::DecodeControl {
                path: path.to_owned(),
                cause,
            })?
    }

    fn resolve(
        &mut self,
        field: &worth_query_declaration::facade::application_schema::ApplicationFieldBindingLocus,
        value: worth_foundational::facade::AspectValue,
    ) -> Result<ErasedObserved, FragmentDenial> {
        self.admit_decision_target(&worth_query_declaration::facade::application_schema::ApplicationOperationDecisionReadTarget::Field { entity: field.entity().to_owned(), aspect: field.aspect().to_owned(), field: field.field().to_owned() }).map_err(HandlerExecutionDenial::new)?;
        let identity = self
            .reader
            .resolve_encoded_entity::<()>(field, value)
            .map_err(FragmentDenial::Resolution)?;
        self.tracked_body_locator(&identity, field)
            .map_err(HandlerExecutionDenial::new)?;
        Ok(ErasedObserved {
            identity: identity.fragment_schema::<()>(),
        })
    }

    fn record(
        &mut self,
        target: &ErasedObserved,
        aspect: &AspectDefinition,
    ) -> Result<Option<BTreeMap<String, worth_foundational::facade::AspectValue>>, FragmentDenial>
    {
        let identity = target.identity.fragment_schema::<S>();
        let first = aspect
            .fields()
            .first()
            .ok_or_else(|| missing(aspect.path()))?;
        let locator = self
            .tracked_body_locator(&identity, first.recipe().locus())
            .map_err(HandlerExecutionDenial::new)?;
        let key = WorthQueryApplicationFactKey::AspectPresence {
            entity: aspect.entity().to_owned(),
            entity_id: identity.entity_id,
            locator: locator.clone(),
        };
        self.decision_facts
            .record(key.clone(), DecisionReadOutcome::Observed);
        if !self.reader.work_budget.consume(1) {
            return Err(FragmentDenial::Read(HandlerExecutionDenial::new(WorthQueryApplicationAttemptDenial::from(worth_relational::facade::transactions::RelationalExecutionDenialCause::WorkExhausted))));
        }
        self.reader.realized_scope.record(identity.entity_id);
        let present = observe_aspect_presence(
            self.reader.runtime,
            self.reader.execution,
            self.reader.snapshot,
            identity.entity_id,
            identity.kind,
            locator.aspect().aspect_key(),
        );
        let present = match present {
            Ok(Some(present)) => present,
            Ok(None) => {
                self.decision_facts.record(key, DecisionReadOutcome::Failed);
                return Err(missing(aspect.path()));
            }
            Err(cause) => {
                self.reader.native_read_stop.get_or_insert(cause);
                self.decision_facts.record(key, DecisionReadOutcome::Failed);
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
                self.tracked_body_locator(&identity, locus)
                    .map_err(HandlerExecutionDenial::new)?
            };
            let path = format!("{}.{}", aspect.path(), locus.field());
            let decoder = self.reader.value_decode;
            let raw = self
                .reader
                .body_scalar_observe(&identity, &locator, |raw| {
                    raw.map(|raw| {
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
                            HandlerExecutionDenial::new(WorthQueryApplicationAttemptDenial::new(
                                WorthQueryApplicationAttemptDenialKind::ValueDecode(cause),
                                &path,
                            ))
                        })?;
                        Ok(raw.clone())
                    })
                    .transpose()
                })?;
            if let Some(raw) = raw.filter(|_| !field.internal_presence()) {
                values.insert(path, raw);
            }
        }
        Ok(present.then_some(values))
    }

    fn target(
        &self,
        target: &ErasedObserved,
    ) -> Result<super::super::WorthQueryInvariantMutationTarget<(), ()>, FragmentDenial> {
        let identity = target.identity.fragment_schema::<S>();
        let target = self
            .mutation_target(&identity)
            .map_err(HandlerExecutionDenial::new)?;
        Ok(target.fragment_schema::<()>())
    }
}

fn missing(path: &str) -> FragmentDenial {
    FragmentDenial::Read(missing_handler(path))
}
fn missing_handler(path: &str) -> HandlerExecutionDenial {
    HandlerExecutionDenial::new(WorthQueryApplicationAttemptDenial::new(
        WorthQueryApplicationAttemptDenialKind::MissingAuthoritativeFact,
        path,
    ))
}
