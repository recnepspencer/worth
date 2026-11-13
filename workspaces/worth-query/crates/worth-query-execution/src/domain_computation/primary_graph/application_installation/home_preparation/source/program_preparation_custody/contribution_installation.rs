//! Contributor installation keeps every advanced original in its refusal owner.
use super::*;
impl<S: ApplicationSchemaComposition, P> HomeProgramInstallationCustody<S, P> {
    pub(in crate::domain_computation::primary_graph::application_installation::home_preparation::source) fn install_contributions(
        &mut self,
        phase: &crate::domain_computation::primary_graph::WorthQueryBootstrapAdvancementPhase<'_>,
    ) -> Result<(), D> {
        let bootstrap = self
            .bootstrap
            .as_mut()
            .expect("actual bootstrap before conditional construction");
        let meaning = self
            .meaning
            .as_ref()
            .expect("actual meaning before publication construction");
        match self
            .pending_producers
            .take()
            .expect("one original producer batch")
            .seal_with_support(
                self.core.build,
                bootstrap.resource_support.snapshot_ref(),
                meaning.runtime.installed_packages(),
            ) {
            Ok(producers) => self.producers = Some(producers),
            Err(refused) => {
                let cause = refused.reason().clone();
                self.producer_refusal = Some(refused);
                return Err(D::Producer(cause));
            }
        }
        let producers = self.producers.as_ref().expect("same sealed producer batch");
        crate::domain_computation::primary_graph::application_installation::source_scope_ownership::seal(&bootstrap.graph.layout,&mut bootstrap.graph.application_publication,producers).map_err(D::PrimaryGraph)?;
        let RuntimeMeaning {
            runtime,
            authority,
            installed,
        } = self.meaning.take().expect("one original runtime meaning");
        let publication=ApplicationRuntimePublication {bootstrap:*self.bootstrap.take().expect("same bootstrap"),runtime,authority,installed_schema:installed,authorization_clock:self.core.clock.take().expect("same original clock"),fault_port:crate::domain_computation::primary_graph::provider::fault_port::production_fault_port(),conditional_evaluation_budget:self.core.limits.conditionals};
        match WorthQueryConditionalApplicationRuntimeInstallation::new(phase, publication) {
            Ok(installation) => self.conditional = Some(Box::new(installation)),
            Err(refused) => {
                let (publication, cause) = refused.into_parts();
                self.publication = Some(Box::new(publication));
                return Err(D::Conditional(Box::new(cause)));
            }
        }
        let installation = self
            .conditional
            .as_mut()
            .expect("actual conditional installation");
        installation.install_output_producers(producers.clone());
        match self
            .pending_conditionals
            .take()
            .expect("one original conditional batch")
            .install_all(producers, installation)
        {
            Ok(conditionals) => self.conditionals = Some(conditionals),
            Err(refused) => {
                let cause = D::Conditional(Box::new(refused.reason().clone()));
                self.conditional_refusal = Some(refused);
                return Err(cause);
            }
        }
        self.parts = Some(
            self.conditional
                .take()
                .expect("same installation")
                .into_home_program_parts(),
        );
        let parts = self
            .parts
            .take()
            .expect("same supported installation parts");
        self.temporal_pending = parts.bindings;
        self.readiness = parts.output_readiness;
        self.definition_pending = parts.definitions;
        let ApplicationRuntimePublication {
            bootstrap,
            runtime,
            authority,
            installed_schema,
            authorization_clock,
            ..
        } = parts.publication;
        self.bootstrap = Some(Box::new(bootstrap));
        self.meaning = Some(RuntimeMeaning {
            runtime,
            authority,
            installed: installed_schema,
        });
        self.core.clock = Some(authorization_clock);
        Ok(())
    }
    pub(in crate::domain_computation::primary_graph::application_installation::home_preparation::source) fn install_temporal_bindings(
        &mut self,
    ) -> Result<(), D> {
        let meaning = self
            .meaning
            .as_ref()
            .expect("same installed runtime meaning");
        let services = self.services.as_ref().expect("same prepared services");
        let cursor = services
            .provider
            .graph
            .source_owner
            .invalidation_owner
            .latest_position();
        self.operations = Some(crate::domain_computation::primary_graph::conditional_operation::install_pending_bindings(
            std::mem::take(&mut self.temporal_pending),
            self.builder.as_mut().expect("same authored programme builder"),
            &services.participation,
            cursor,
            meaning.runtime.authority_identity().as_u64(),
            meaning.runtime.installed_packages().runtime_ordinal(),
            meaning.runtime.installed_packages().generation().ordinal(),
            services.participation.provider_identity(),
            crate::domain_computation::primary_graph::application_branch::PRIMARY_APPLICATION_BRANCH,
        ).map_err(|cause| D::Conditional(Box::new(cause)))?);
        Ok(())
    }
    pub(in crate::domain_computation::primary_graph::application_installation::home_preparation::source) fn prepare_services(
        &mut self,
        phase: &crate::domain_computation::primary_graph::WorthQueryBootstrapAdvancementPhase<'_>,
    ) -> Result<(), D> {
        let meaning = self.meaning.as_ref().expect("readmitted original meaning");
        self.services=Some(crate::domain_computation::primary_graph::application_runtime::installation::home_attachment::prepare_home_application_services(phase,self.bootstrap.as_mut().expect("actual bootstrap"),&meaning.runtime,&meaning.authority,&meaning.installed).map_err(D::PrimaryGraph)?);
        Ok(())
    }
}
