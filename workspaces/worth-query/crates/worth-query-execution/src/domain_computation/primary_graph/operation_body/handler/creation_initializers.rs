//! One initializer owner serves root, peer and owned-child sites.
use super::super::{value::PreparedBodyValues, PreparedBodySource};
use super::{ApplicationMutationBinding, ApplicationSchema, CandidateWriter};
use crate::domain_computation::primary_graph::{
    WorthQueryApplicationAttemptDenial, WorthQueryApplicationEffectEntity,
};
use worth_foundational::facade::AspectValue;
use worth_query_declaration::facade::application_schema::{
    ApplicationOperationBodyCreatedField, ApplicationOperationBodyCreation,
};
pub(super) type InputValues<'a> = Option<std::vec::Drain<'a, Option<AspectValue>>>;
pub(super) fn initialize<S: ApplicationSchema, B: ApplicationMutationBinding<S>>(
    creation: &ApplicationOperationBodyCreation,
    created: &WorthQueryApplicationEffectEntity<S, ()>,
    inputs: &mut InputValues<'_>,
    sources: &mut std::vec::IntoIter<Option<PreparedBodySource>>,
    values: Option<&PreparedBodyValues>,
    writer: &mut CandidateWriter<'_, S, B>,
) -> Result<(), WorthQueryApplicationAttemptDenial> {
    for step in creation.fields() {
        let mut source_reservation = None;
        let value = match step {
            ApplicationOperationBodyCreatedField::Computed { value, .. } => {
                std::borrow::Cow::Borrowed(values.expect("installed computed value").value(value))
            }
            ApplicationOperationBodyCreatedField::Literal { value, .. } => {
                std::borrow::Cow::Borrowed(value)
            }
            ApplicationOperationBodyCreatedField::Input { .. } => std::borrow::Cow::Owned(
                inputs
                    .as_mut()
                    .and_then(Iterator::next)
                    .expect("one prepared value per installed input initializer")
                    .expect("required initializer input is present"),
            ),
            ApplicationOperationBodyCreatedField::OptionalInput { .. } => {
                let Some(value) = inputs
                    .as_mut()
                    .and_then(Iterator::next)
                    .expect("one prepared option per installed optional initializer")
                else {
                    continue;
                };
                std::borrow::Cow::Owned(value)
            }
            ApplicationOperationBodyCreatedField::ScopeField { .. }
            | ApplicationOperationBodyCreatedField::PresentScopeField { .. }
            | ApplicationOperationBodyCreatedField::CapturedScopeField { .. }
            | ApplicationOperationBodyCreatedField::CapturedPathField { .. }
            | ApplicationOperationBodyCreatedField::RelatedField { .. }
            | ApplicationOperationBodyCreatedField::PrincipalIdentity { .. } => {
                let Some(source) = sources
                    .next()
                    .expect("one captured source per installed initializer")
                else {
                    continue;
                };
                let (source, reservation) = source.into_parts();
                source_reservation = Some(reservation);
                std::borrow::Cow::Owned(source)
            }
            ApplicationOperationBodyCreatedField::SigningProvenance { .. } => continue,
        };
        writer.body_initialize_field(created, step.field(), value)?;
        drop(source_reservation);
    }

    Ok(())
}
