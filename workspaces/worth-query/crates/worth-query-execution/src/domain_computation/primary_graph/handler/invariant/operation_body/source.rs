//! Capture source values once from sealed scope/principal/related identities before candidate construction.
use super::{
    ApplicationMutationBinding, ApplicationSchema, DecisionReader, HandlerExecutionDenial,
};
use crate::domain_computation::primary_graph::{
    operation_body::{PreparedBodySelections, PreparedBodySource},
    HandlerResult, WorthQueryApplicationAttemptDenial, WorthQueryApplicationAttemptDenialKind,
    WorthQueryOperationBodyDenial,
};
use worth_query_declaration::facade::application_schema::{
    ApplicationFieldPresence, ApplicationOperationBodyCreatedField, ApplicationOperationBodyProgram,
};

impl<Schema: ApplicationSchema, Binding: ApplicationMutationBinding<Schema>>
    DecisionReader<'_, '_, '_, Schema, Binding>
{
    pub(in crate::domain_computation::primary_graph) fn body_created_sources(
        &mut self,
        body: &ApplicationOperationBodyProgram,
        selections: &PreparedBodySelections<Schema>,
    ) -> HandlerResult<Vec<Option<PreparedBodySource>>, WorthQueryOperationBodyDenial> {
        let mut sources = Vec::new();
        let Some(creation) = body.creation() else {
            return HandlerResult::Completed(sources);
        };
        for step in creation.fields() {
            let (source, captured) = if let Some(path) = step.path_source() {
                let selected = self
                    .execution
                    .decision_stage(|context| {
                        self.reader.body_path_selection(
                            self.scope,
                            &path.path,
                            self.execution,
                            context,
                        )
                    })
                    .map_err(HandlerExecutionDenial::new);
                let captured = match selected {
                    Ok(Ok(Some(selected))) => {
                        self.body_related_value(&path.field, &selected.identity)
                    }
                    Ok(Ok(None)) => Err(HandlerExecutionDenial::new(
                        WorthQueryApplicationAttemptDenial::new(
                            WorthQueryApplicationAttemptDenialKind::MissingAuthoritativeFact,
                            path.path.terminal_entity(),
                        ),
                    )),
                    Ok(Err(cause)) | Err(cause) => Err(cause),
                };
                (&path.field, captured)
            } else if let Some(related) = step.related_source() {
                let Some(selected) =
                    selections.selected(&related.relation, &related.from, &related.to)
                else {
                    return HandlerResult::ExecutionDenied(HandlerExecutionDenial::new(
                        WorthQueryApplicationAttemptDenial::new(
                            WorthQueryApplicationAttemptDenialKind::MissingAuthoritativeFact,
                            &related.relation,
                        ),
                    ));
                };
                (
                    &related.field,
                    self.body_related_value(&related.field, &selected.identity),
                )
            } else if let Some(source) = step.source_field() {
                (source, self.body_scope_value(source))
            } else if let Some(source) = step.principal_source() {
                (source, self.body_principal_value(source))
            } else {
                continue;
            };
            match captured {
                Ok(Some(value)) => sources.push(Some(value)),
                Ok(None) if matches!(step, ApplicationOperationBodyCreatedField::RelatedField { source, .. } if source.presence == ApplicationFieldPresence::Optional) => {
                    sources.push(None)
                }
                Ok(None) if matches!(step, ApplicationOperationBodyCreatedField::CapturedPathField { source, target_presence: ApplicationFieldPresence::Optional, .. } if source.presence == ApplicationFieldPresence::Optional) => {
                    sources.push(None)
                }
                Ok(None) => {
                    return HandlerResult::DomainDenied(
                        WorthQueryOperationBodyDenial::SourceMismatch {
                            entity: source.entity().into(),
                            aspect: source.aspect().into(),
                            field: source.field().into(),
                        },
                    )
                }
                Err(cause) => return HandlerResult::ExecutionDenied(cause),
            }
        }
        HandlerResult::Completed(sources)
    }
}
