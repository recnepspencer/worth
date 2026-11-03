//! The Query publication boundary separates request stops from ordinary refusals.
use worth_runtime_world::facade::{
    NoEffectCompositePublication, RuntimeWorldSettledRelationalAdoptionDenial,
};
#[derive(Debug)]
pub(crate) enum WorthQueryProductPublicationDenial<Refusal> {
    Stopped(worth_execution::WorkCeilingDenial),
    Refused(Refusal),
}
#[derive(Debug)]
pub(crate) enum WorthQueryProductPublicationOutcome<
    Performed = worth_runtime_world::facade::PerformedCompositePublication,
    Unpublished = worth_runtime_world::facade::ProductUnpublishedOwnerEffects,
> {
    Performed(Performed),
    ProductUnpublished(Unpublished),
}
impl WorthQueryProductPublicationDenial<NoEffectCompositePublication> {
    pub(super) fn publication(no_effect: NoEffectCompositePublication) -> Self {
        match crate::domain_computation::primary_graph::publication_request_denial::publication(
            no_effect.cause(),
        ) {
            Err(stop) => Self::Stopped(stop),
            Ok(_) => Self::Refused(no_effect),
        }
    }
}
impl WorthQueryProductPublicationDenial<RuntimeWorldSettledRelationalAdoptionDenial> {
    pub(super) fn adoption(denial: RuntimeWorldSettledRelationalAdoptionDenial) -> Self {
        match &denial {
            RuntimeWorldSettledRelationalAdoptionDenial::Publication(no_effect) => {
                match crate::domain_computation::primary_graph::publication_request_denial::publication(no_effect.cause()) {
                    Err(stop) => Self::Stopped(stop),
                    Ok(_) => Self::Refused(denial),
                }
            }
            RuntimeWorldSettledRelationalAdoptionDenial::Recovery(_) => Self::Refused(denial),
        }
    }
}
