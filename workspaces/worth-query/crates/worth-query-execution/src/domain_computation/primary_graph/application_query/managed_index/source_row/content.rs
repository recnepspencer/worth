//! Typed selection is a thin shell; Native frame certification is concrete.
use super::*;

use worth_query_declaration::facade::application_schema::ApplicationContentBinding;

impl<S: ApplicationSchema, I: ApplicationManagedIndexBinding<S>>
    WorthQueryManagedIndexSourceRow<'_, S, I>
{
    /// Lend the exact canonical payload of this installed Source field.
    pub fn with_content<FieldSlot, Entity, Aspect, Field, C, Output>(
        &self,
        selector: ApplicationQueryResultFieldRef<Query<S, I>, FieldSlot, S, Field>,
        _format: C,
        body: impl FnOnce(&[u8]) -> Output,
    ) -> Result<Output, WorthQueryManagedIndexContentReadDenial>
    where
        C: ApplicationContentBinding,
        Field: worth_query_declaration::facade::application_schema::fragment::RecordSelection<
            Entity = Entity,
            Aspect = Aspect,
        >,
        Field::Binding:
            worth_query_declaration::facade::application_schema::ApplicationContentReferenceBinding,

        FieldSlot: WorthQueryPortableType,
    {
        self.check()?;
        super::super::source_content::admit_field(
            self.control,
            &[
                selector.entity(),
                selector.aspect(),
                selector.field(),
                selector.output_name(),
                Query::<S, I>::QUERY_TYPE_IDENTITY.as_str(),
                FieldSlot::PORTABLE_TYPE_IDENTITY.as_str(),
                Field::Binding::IDENTITY.as_str(),
                C::NAME,
            ],
        )?;
        let projected = self.row.required_field(selector).map_err(|cause| {
            self.control.refuse(Stop::Projection(cause.clone()));
            WorthQueryManagedIndexContentReadDenial::Projection(cause)
        })?;
        let binding = self
            .decode
            .content_binding::<C>(selector.entity(), selector.aspect(), selector.field())
            .map_err(|_| self.decode_stop())?;
        let frame = admitted(projected, &binding, &self.execution, self.control)?;
        let output = body(frame.payload());
        self.check()?;
        Ok(output)
    }
}

use super::super::source_content::admitted;
pub use super::super::source_content::WorthQueryManagedIndexContentReadDenial;
