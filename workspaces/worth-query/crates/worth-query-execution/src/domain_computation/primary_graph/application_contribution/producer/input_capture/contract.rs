//! Stable input meaning is declared separately from its configured handler type.
use super::*;
use crate::equality_type_id::EqualityTypeId;
use worth_query_declaration::facade::application_program::{
    ApplicationConnectionDeclaration, ApplicationOccurrenceConnections,
};

/// Names a producer's pure input preparation and its complete finite parent ports.
pub struct WorthQueryProducerInputCaptureContract<S> {
    pub(in crate::domain_computation::primary_graph::application_contribution) meaning:
        InputCaptureMeaning,
    marker: PhantomData<fn() -> S>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(in crate::domain_computation::primary_graph) struct InputCaptureMeaning {
    pub(in crate::domain_computation::primary_graph) definition: EqualityTypeId,
    pub(in crate::domain_computation::primary_graph) identity: &'static str,
    pub(in crate::domain_computation::primary_graph) family: EqualityTypeId,
    pub(in crate::domain_computation::primary_graph) ports: Vec<ApplicationConnectionDeclaration>,
    pub(in crate::domain_computation::primary_graph) types: Vec<std::any::TypeId>,
}

impl<S: ApplicationSchema> WorthQueryProducerInputCaptureContract<S> {
    pub fn new<C: WorthQueryProducerInputCaptureDefinition<S>>() -> Self {
        let mut ports = Vec::new();
        let mut types = Vec::new();
        C::Parents::append(&mut ports);
        C::Parents::append_types(&mut types);
        Self {
            meaning: InputCaptureMeaning {
                definition: EqualityTypeId::of::<C>(),
                identity: C::IDENTITY,
                family: EqualityTypeId::of::<C::OutputFamily>(),
                ports,
                types,
            },
            marker: PhantomData,
        }
    }
}

impl InputCaptureMeaning {
    pub(in crate::domain_computation::primary_graph::application_contribution) fn validate<S, P>(
        &self,
    ) -> bool
    where
        S: ApplicationSchema,
        P: WorthQueryApplicationProducerBinding<S>,
    {
        self.family == EqualityTypeId::of::<P::OutputFamily>()
            && <P::Operation as ApplicationMutationBinding<S>>::REQUIRES_APPLICATION_PROGRAM
            && !self.identity.is_empty()
            && !self.ports.is_empty()
            && self.ports.len() == self.types.len()
            && self.ports.iter().enumerate().all(|(i, port)| {
                port.input_capture_type().is_none()
                    && port.occurrence_component().is_none()
                    && !port.is_demand_only_root()
                    && !self.ports[..i].contains(port)
                    && !self.types[..i].contains(&self.types[i])
            })
    }

    /// Point TypeIds stay out of the portable producing-edition identity.
    pub(in crate::domain_computation::primary_graph::application_contribution) fn portable_meaning(
        &self,
    ) -> (
        &'static str,
        Vec<(
            &'static str,
            &'static str,
            &'static str,
            &'static str,
            &'static str,
            &'static str,
            &'static str,
            bool,
            bool,
        )>,
    ) {
        (
            self.identity,
            self.ports
                .iter()
                .map(|port| {
                    (
                        port.identity(),
                        port.source_instance(),
                        port.source_feature(),
                        port.source_port(),
                        port.target_instance(),
                        port.target_feature(),
                        port.target_port(),
                        port.target_required(),
                        port.exports_across_instances(),
                    )
                })
                .collect(),
        )
    }
}
