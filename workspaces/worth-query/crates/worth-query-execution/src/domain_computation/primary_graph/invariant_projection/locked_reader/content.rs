//! Content references retain field absence evidence instead of collapsing it into Option.
use super::WorthQueryApplicationInvariantProjectionReader as Reader;
use {crate::domain_computation::primary_graph::vocabulary::application_attempt::predecode_admission::observe_field_observation_borrowed_admitted, crate::domain_computation::vocabulary::primary_graph::value_decode::WorthQueryValueDecodeDenial as Stop};
use worth_foundational::facade::{AspectFieldLocator, ContentRefId};
use worth_query_declaration::facade::application_schema::ContentRefApplicationValueBinding;
use worth_query_installation::facade::ApplicationSchema;
use worth_relational::facade::{
    identity::{EntityId, KindId},
    runtime::RelationalProjectedFieldObservation as Raw,
};
impl<S: ApplicationSchema> Reader<'_, S> {
    pub(in crate::domain_computation::primary_graph::invariant_projection) fn content_reference(
        &mut self,
        entity: EntityId,
        kind: KindId,
        locus: (&str, &str, &str),
        locator: &AspectFieldLocator,
    ) -> Result<Option<ContentRefId>, Stop> {
        ContentRead {
            runtime: self.runtime,
            execution: self.execution,
            snapshot: self.snapshot,
            decode: self.value_decode,
            work: &mut self.work,
            budget: &mut self.work_budget,
            scope: &mut self.realized_scope,
            native_stop: &mut self.native_read_stop,
        }
        .run(ContentFieldRead {
            entity,
            kind,
            locus,
            locator,
            optional_field: self.layout.field_is_optional(locus.0, locator),
            optional_aspect: self.layout.aspect_is_optional(locus.0, locator),
        })
    }
}
struct ContentRead<'borrow, 'run, 'host> {
    runtime: &'borrow worth_relational::facade::runtime::RelationalRuntime,
    execution: &'borrow worth_execution::ExecutionRequest<'run, 'host>,
    snapshot: &'borrow worth_relational::facade::snapshots::SnapshotHandle,
    decode: &'borrow crate::domain_computation::primary_graph::value_decode::ValueDecodeSession<
        'borrow,
    >,
    work: &'borrow mut super::super::WorthQueryInvariantProjectionWork,
    budget: &'borrow mut super::super::work::WorthQueryInvariantProjectionWorkBudget,
    scope: &'borrow mut super::super::WorthQueryRealizedProjectionScope,
    native_stop:
        &'borrow mut Option<worth_relational::facade::transactions::RelationalExecutionDenialCause>,
}
impl ContentRead<'_, '_, '_> {
    fn run(self, field: ContentFieldRead<'_>) -> Result<Option<ContentRefId>, Stop> {
        let decode = self.decode;
        let result = self.read(field);
        if let Err(cause) = result {
            decode.refuse_content(cause);
        }
        result
    }
    fn read(self, field: ContentFieldRead<'_>) -> Result<Option<ContentRefId>, Stop> {
        if let Some(cause) = self.decode.denial() {
            return Err(cause);
        }
        if let Some(cause) = *self.native_stop {
            return Err(Stop::Content(cause.into()));
        }
        if !self.budget.consume(1) {
            return Err(if self.budget.overflowed() {
                Stop::ContentProjectionWorkOverflow
            } else {
                Stop::ContentProjectionWorkExceeded
            });
        }
        self.scope.record(field.entity);
        read_reference(
            self.runtime,
            self.execution,
            self.snapshot,
            self.decode,
            self.work,
            field,
        )
        .inspect_err(|cause| {
            if let Stop::Content(
                worth_relational::facade::content::RelationalContentDenial::Execution(cause),
            ) = cause
            {
                self.native_stop.get_or_insert(*cause);
            }
        })
    }
}

struct ContentFieldRead<'a> {
    entity: EntityId,
    kind: KindId,
    locus: (&'a str, &'a str, &'a str),
    locator: &'a AspectFieldLocator,
    optional_field: bool,
    optional_aspect: bool,
}
// One concrete observation engine is shared by every application's typed wrapper.
fn read_reference(
    runtime: &worth_relational::facade::runtime::RelationalRuntime,
    execution: &worth_execution::ExecutionRequest<'_, '_>,
    snapshot: &worth_relational::facade::snapshots::SnapshotHandle,
    decode: &crate::domain_computation::primary_graph::value_decode::ValueDecodeSession<'_>,
    work: &mut super::super::WorthQueryInvariantProjectionWork,
    field: ContentFieldRead<'_>,
) -> Result<Option<ContentRefId>, Stop> {
    let (name, aspect, key) = field.locus;
    let observed = observe_field_observation_borrowed_admitted(
        runtime,
        execution,
        snapshot,
        field.entity,
        field.kind,
        field.locator,
        |raw| {
            work.record_fields(1);
            match raw {
                Raw::Present(value) => decode
                    .decode::<ContentRefApplicationValueBinding>(name, aspect, key, value)
                    .map(Some)
                    .ok_or_else(|| match decode.denial() {
                        Some(cause) => cause,
                        None => Stop::Content(worth_relational::facade::content::RelationalContentDenial::EncodingInconsistency),
                    }),
                Raw::AspectAbsent if field.optional_aspect => Ok(None),
                Raw::FieldAbsent if field.optional_field => Ok(None),
                Raw::AspectAbsent | Raw::FieldAbsent => Err(Stop::RequiredContentFieldAbsent),
                Raw::UnsupportedAspectShape => Err(Stop::BindingContractMismatch),
            }
        },
    )
    .map_err(|cause| Stop::Content(cause.into()))?;
    observed.ok_or(Stop::ContentBasisUnavailable)?
}
