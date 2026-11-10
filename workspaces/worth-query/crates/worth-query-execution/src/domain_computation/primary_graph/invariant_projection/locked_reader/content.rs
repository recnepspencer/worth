//! Content references retain field absence evidence instead of collapsing it into Option.
use super::super::read_basis::NativeProjectionRead;
use super::WorthQueryApplicationInvariantProjectionReader as Reader;
use crate::domain_computation::primary_graph::content::{ContentFieldSelection, DecodeReference};
use crate::domain_computation::primary_graph::WorthQueryValueDecodeDenial as Stop;
use worth_foundational::facade::{AspectFieldLocator, ContentRefId};
use worth_query_declaration::facade::application_schema::ApplicationContentReferenceBinding;
use worth_query_installation::facade::ApplicationSchema;
use worth_relational::facade::{
    identity::{EntityId, KindId},
    runtime::RelationalProjectedFieldObservation as Raw,
};
impl<S: ApplicationSchema> Reader<'_, S> {
    fn content_reference(
        &mut self,
        entity: EntityId,
        kind: KindId,
        locus: (&str, &str, &str),
        locator: &AspectFieldLocator,
        decode: DecodeReference,
    ) -> Result<Option<ContentRefId>, Stop> {
        ContentRead {
            native: &self.native,
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
            decode,
            optional_field: self.layout.field_is_optional(locus.0, locator),
            optional_aspect: self.layout.aspect_is_optional(locus.0, locator),
        })
    }
}
struct ContentRead<'borrow, 'runtime, 'run, 'host> {
    native: &'borrow NativeProjectionRead<'runtime>,
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
impl ContentRead<'_, '_, '_, '_> {
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
            self.native,
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
    decode: DecodeReference,
    optional_field: bool,
    optional_aspect: bool,
}
// One concrete observation engine is shared by every application's typed wrapper.
fn read_reference(
    native: &NativeProjectionRead<'_>,
    execution: &worth_execution::ExecutionRequest<'_, '_>,
    snapshot: &worth_relational::facade::snapshots::SnapshotHandle,
    decode: &crate::domain_computation::primary_graph::value_decode::ValueDecodeSession<'_>,
    work: &mut super::super::WorthQueryInvariantProjectionWork,
    field: ContentFieldRead<'_>,
) -> Result<Option<ContentRefId>, Stop> {
    let (name, aspect, key) = field.locus;
    let observed = native.observe_field(
        execution,
        snapshot,
        field.entity,
        field.kind,
        field.locator,
        |raw| {
            work.record_fields(1);
            match raw {
                Raw::Present(value) => (field.decode)(decode, name, aspect, key, value)
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

impl<S: ApplicationSchema> Reader<'_, S> {
    /// Resolves one field's immutable payload; work is proportional to the field
    /// lookup and admitted frame, never the graph or its history.
    pub fn content<E, F, C>(
        &mut self,
        identity: &crate::domain_computation::primary_graph::WorthQueryInvariantEntityIdentity<
            S,
            E,
        >,
        content: worth_query_declaration::facade::application_schema::ApplicationContentFieldRef<
            F,
            C,
        >,
    ) -> Result<
        Option<crate::domain_computation::primary_graph::AdmittedContent<C>>,
        crate::domain_computation::primary_graph::ContentReadDenial,
    >
    where
        F: worth_query_declaration::facade::application_schema::fragment::RecordSelection<
            Entity = E,
        >,
        F::Binding: ApplicationContentReferenceBinding,
        C: worth_query_declaration::facade::application_schema::ApplicationContentBinding,
    {
        crate::domain_computation::primary_graph::content::typed(self.content_frame(
            identity.native_identity(),
            ContentFieldSelection::of::<F, C>(content.field()),
        ))
    }

    fn content_frame(
        &mut self,
        identity: super::super::native_read::NativeIdentity<'_>,
        selection: ContentFieldSelection,
    ) -> Result<
        Option<worth_relational::facade::content::RelationalContentFrame>,
        crate::domain_computation::primary_graph::ContentReadDenial,
    > {
        use crate::domain_computation::primary_graph::{content, ContentReadDenial as Denial};
        let field = selection.column;
        let session = self.value_decode;
        if !identity.local_to(
            self.authority_identity,
            self.native
                .candidate_identity()
                .map(|id| id.qualification()),
            field.entity(),
        ) {
            session.refuse_content(Stop::BindingContractMismatch);
            return Err(Denial::BindingContractMismatch);
        }
        self.layout.record_column(field).map_err(|_| {
            session.refuse_content(Stop::BindingContractMismatch);
            Denial::BindingContractMismatch
        })?;
        let binding = session.content_binding_matching(
            field.entity(),
            field.aspect(),
            field.field(),
            selection.matches,
        )?;
        let locator = self
            .layout
            .field_locator(field.entity(), field.aspect(), field.field())
            .ok_or_else(|| {
                session.refuse_content(Stop::BindingContractMismatch);
                Denial::BindingContractMismatch
            })?
            .clone();
        let Some(value) = self.content_reference(
            identity.entity_id,
            identity.kind,
            (field.entity(), field.aspect(), field.field()),
            &locator,
            selection.decode,
        )?
        else {
            return Ok(None);
        };
        content::resolve_frame(self.execution, session, &binding, value, &mut || {
            self.native
                .projection(self.snapshot)
                .ok_or(Denial::UnavailableBasis)?
                .retain_entity_field_content(
                    self.execution,
                    identity.entity_id,
                    identity.kind,
                    &locator,
                )
                .map_err(Denial::Content)?
                .ok_or(Denial::UnavailableBasis)
        })
        .map(Some)
    }
}
