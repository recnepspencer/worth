//! A declared original-C read list, never a predecessor route or read grant.
use super::super::WorthQueryApplicationProducerBinding;
use super::*;
pub struct ProducerOriginalReadAccess {
    require: fn(&str, &str, EqualityTypeId) -> Result<(), WorthQueryProducerObservationReadDenial>,
}
impl ProducerOriginalReadAccess {
    pub fn for_binding<S, B>() -> Option<Self>
    where
        S: ApplicationSchema,
        B: WorthQueryApplicationProducerBinding<S>,
    {
        B::ORIGINAL_READS?;
        Some(Self {
            require: require::<S, B>,
        })
    }
    pub fn require<S, O>(&self) -> Result<(), WorthQueryProducerObservationReadDenial>
    where
        S: ApplicationSchema,
        O: ApplicationOperationMarkerIdentity<S> + 'static,
    {
        (self.require)(
            O::IDENTIFIER,
            O::InputBinding::IDENTITY_NAME,
            EqualityTypeId::of::<O>(),
        )
    }
}
fn require<S, B>(
    name: &str,
    input: &str,
    operation: EqualityTypeId,
) -> Result<(), WorthQueryProducerObservationReadDenial>
where
    S: ApplicationSchema,
    B: WorthQueryApplicationProducerBinding<S>,
{
    B::ORIGINAL_READS
        .ok_or(WorthQueryProducerObservationReadDenial::UndeclaredObservationRead)?
        .require_identity(name, input, operation)
}
