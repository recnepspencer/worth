//! Rebind historical DATA through actual installed declarations and membership.
use super::super::disclosure::WorthQueryCorrectionDerivationDenial as Denial;
use super::*;
use crate::domain_computation::primary_graph::source_disclosure::OriginalSourceDisclosure;
use worth_query_admission::facade::application_query::readmit_recorded_application_query_parameters;
use worth_query_declaration::facade::application_operation::ApplicationMutationOutputPosture as Posture;
use worth_query_declaration::facade::{
    application_aftermath::{ApplicationCorrectionBinding, DeclaredCorrectionBinding},
    application_operation::ApplicationMutationBinding,
    application_schema::ApplicationSchema,
};

impl HomeCorrectionOccurrence {
    pub(in crate::domain_computation::primary_graph::product_operation::correction_source) fn readmit<
        S,
        C,
    >(
        application: &WorthQueryPrimaryGraphApplicationRuntime<S>,
        material: RetainedCorrectionMaterial,
        original: &worth_runtime_world::facade::WorldOriginalStateObservation<'_>,
        causality: Option<OriginalCorrectionLineage>,
        request: &worth_execution::ExecutionRequest<'_, '_>,
    ) -> Result<Arc<Self>, Denial>
    where
        S: ApplicationSchema,
        C: ApplicationCorrectionBinding<S>,
    {
        let capture = &material.capture;
        if capture.mutation != C::Source::IDENTITY
            || capture.original != original.attempt()
            || &capture.program != original.program_revision().bytes()
        {
            return Err(Denial::UnsupportedSourceMeaning);
        }
        let program = application
            .installed_program_support()
            .ok_or(Denial::UnsupportedSourceMeaning)?
            .readmit_original_meaning(original)
            .map_err(|_| Denial::UnsupportedSourceMeaning)?;
        program
            .action_inventory()
            .resolve_unique_binding::<S, C::Source>()
            .map_err(Denial::Action)?;
        let binding = application
            .installed_schema()
            .installed_mutation_binding::<C::Source>()
            .map_err(|_| Denial::UnsupportedSourceMeaning)?;
        let association = binding
            .retain_correction_binding()
            .ok_or(Denial::UnsupportedSourceMeaning)?;
        if DeclaredCorrectionBinding::of::<S, C>().as_ref() != Ok(association.declaration()) {
            return Err(Denial::UnsupportedSourceMeaning);
        }
        let aftermath = binding
            .operation()
            .contracts()
            .aftermath()
            .cloned()
            .ok_or(Denial::UnsupportedSourceMeaning)?;
        let disclosure = capture
            .disclosure
            .as_ref()
            .ok_or(Denial::UnsupportedSourceMeaning)?;
        let query = application
            .installed_schema()
            .installed_query_binding::<C::Disclosure>()
            .map_err(|_| Denial::UnsupportedSourceMeaning)?;
        if query.query().identity().as_bytes() != &disclosure.query {
            return Err(Denial::UnsupportedSourceMeaning);
        }
        let parameters = readmit_recorded_application_query_parameters(
            query.query(),
            disclosure.parameters.clone(),
        )
        .map_err(|_| Denial::UnsupportedSourceMeaning)?;
        if parameters.identity().bytes() != &disclosure.identity {
            return Err(Denial::UnsupportedSourceMeaning);
        }
        let disclosure = OriginalSourceDisclosure::readmitted(
            query.query().identity().clone(),
            parameters,
            request,
        )
        .map_err(|_| Denial::SemanticMaterialUnavailable)?;
        let disclosure = OriginalCorrectionDisclosure::from_original(association, disclosure);
        let recorded_scope = capture
            .scope
            .as_ref()
            .ok_or(Denial::UnsupportedSourceMeaning)?;
        let scope = WorthQueryObservedScopeSelector::readmit_recorded(
            &recorded_scope.location.aspect,
            &recorded_scope.location.fields,
            recorded_scope.value.clone(),
            recorded_scope.subject,
            request,
        )
        .map_err(Denial::Material)?;
        let descriptor = C::Source::descriptor();
        let roles = material.ready.roles.iter().map(|role| Ok(crate::domain_computation::primary_graph::application_attempt::effect_program::output_correspondence::WorthQueryCheckpointOutputRole {
            role: role.role.clone(), entity_name: role.entity_name.clone(), entity: role.entity,
            posture: match role.posture { 0 => Posture::Preserve, 1 => Posture::Create, 2 => Posture::Retire, _ => return Err(Denial::UnsupportedSourceMeaning) },
        })).collect::<Result<Vec<_>, Denial>>()?;
        let correspondence = WorthQueryApplicationOutputCorrespondence::from_checkpoint_roles(
            descriptor.binding_type().into(),
            descriptor.output_contract_type().into(),
            descriptor
                .output_roles()
                .iter()
                .filter(|role| role.cardinality().admits_absence())
                .map(|role| role.name()),
            &roles,
            |entity| {
                application
                    .installed_schema()
                    .installed_entity_marker_type(entity)
                    .map(Into::into)
            },
            request,
        )
        .map_err(|_| Denial::SemanticMaterialUnavailable)?;
        output_fields::validate(capture, &aftermath, &correspondence)?;
        Ok(Arc::new(Self {
            material,
            binding: descriptor.binding_type(),
            program,
            aftermath,
            disclosure,
            scope,
            correspondence,
            causality,
        }))
    }
}

mod output_fields;
