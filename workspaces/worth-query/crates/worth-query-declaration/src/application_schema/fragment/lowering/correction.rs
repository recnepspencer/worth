//! A correction member supplies its definition and its existing typed dispatch.
use super::{Builder, Denial, FragmentMember, Lowering};
use crate::application_aftermath::{ApplicationCorrectionBinding, ApplicationCorrectionVisitor};
use crate::application_schema::ApplicationSchema;
#[doc(hidden)]
pub struct CorrectionWitness;
impl<S: ApplicationSchema, Set, C: ApplicationCorrectionBinding<S> + 'static>
    FragmentMember<S, Set, CorrectionWitness> for C
{
    fn lower(
        builder: Builder<S>,
        prefix: &str,
        state: &mut Lowering,
    ) -> Result<Builder<S>, Denial> {
        if !state
            .registrations
            .insert((prefix.to_owned(), std::any::TypeId::of::<C>()))
        {
            return Err(Denial::DuplicateFragmentMember {
                path: format!("{prefix}.{}", std::any::type_name::<C>()),
            });
        }
        Ok(builder.correction::<C>())
    }
    fn visit_correction<Visitor>(
        source: std::any::TypeId,
        visitor: Visitor,
    ) -> Result<Visitor::Output, Visitor>
    where
        S: ApplicationSchema,
        Visitor: ApplicationCorrectionVisitor<S>,
    {
        if source == std::any::TypeId::of::<C::Source>() {
            Ok(
                crate::application_aftermath::dispatch_application_correction::<S, C, Visitor>(
                    visitor,
                ),
            )
        } else {
            Err(visitor)
        }
    }
}
