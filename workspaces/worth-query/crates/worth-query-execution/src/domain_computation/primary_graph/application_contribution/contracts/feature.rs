//! Required slots are checked against actual catalogs, never inferred from counts.
use super::super::feature::{required_producers, validate_pair, WorthQueryRequiredFeatureContract};
use super::*;
use worth_query_declaration::facade::application_program::{
    ApplicationFeatureDeclaration, ApplicationRequiredFeatureRoleDeclaration,
    ApplicationRequiredFeatureRoleKind,
};

impl<Schema: ApplicationSchema> WorthQueryApplicationContributionContracts<Schema> {
    pub fn feature<Contract>(
        &mut self,
    ) -> Result<&mut Self, WorthQueryPrimaryGraphInstallationDenial>
    where
        Contract: WorthQueryRequiredFeatureContract<Schema>,
    {
        validate_pair::<Schema, Contract>()?;
        self.producer::<Contract::InitialProducer>()?;
        self.producer::<Contract::PreserveProducer>()?;
        self.conditional_with_requirements::<Contract::InitialReadiness>(required_producers::<
            Schema,
            Contract::InitialProducer,
            Contract::InitialReadiness,
        >())?;
        self.conditional_with_requirements::<Contract::PreserveReadiness>(required_producers::<
            Schema,
            Contract::PreserveProducer,
            Contract::PreserveReadiness,
        >())
    }
}

impl<Schema: ApplicationSchema> WorthQueryApplicationContractCatalog<Schema> {
    /// Called after program admission, before configuration or bootstrap effects.
    pub(in crate::domain_computation::primary_graph) fn validate_required_features(
        &self,
        features: &[ApplicationFeatureDeclaration],
    ) -> Result<(), WorthQueryPrimaryGraphInstallationDenial> {
        for feature in features {
            for role in feature.required_roles() {
                self.require_feature_role(role)?;
            }
        }
        Ok(())
    }

    fn require_feature_role(
        &self,
        role: &ApplicationRequiredFeatureRoleDeclaration,
    ) -> Result<(), WorthQueryPrimaryGraphInstallationDenial> {
        let producer = self
            .producers
            .get(role.producer())
            .ok_or_else(|| denial(DenialKind::MissingProducerBinding, role.producer()))?;
        let conditional = self
            .conditionals
            .get(role.readiness())
            .ok_or_else(|| denial(DenialKind::MissingConditionalBinding, role.readiness()))?;
        let lifecycle = match role.kind() {
            ApplicationRequiredFeatureRoleKind::Initial => {
                super::super::WorthQueryProducerLifecyclePosture::Initial
            }
            ApplicationRequiredFeatureRoleKind::Preserve => {
                super::super::WorthQueryProducerLifecyclePosture::Preserve
            }
        };
        if producer.binding_type != TypeId::from(role.producer_type())
            || producer.operation_binding_type != TypeId::from(role.mutation_type())
            || producer.operation_type != TypeId::from(role.operation_type())
            || producer.operation != role.mutation()
            || producer.output_family_type != TypeId::from(role.output().family_type())
            || producer.source_type != TypeId::from(role.output().source_type())
            || producer.output_family != role.output().family()
            || producer.source_selector != role.output().source()
            || producer.output_role != role.output().role()
            || producer.applicability.is_empty()
            || producer
                .applicability
                .iter()
                .any(|case| case.lifecycle() != lifecycle)
        {
            return Err(denial(
                DenialKind::ProducerBindingMeaningMismatch,
                role.producer(),
            ));
        }
        if conditional.binding_type != TypeId::from(role.readiness_type())
            || conditional.operation_type != producer.operation_type
            || conditional.owner != producer.owner
            || !conditional
                .required_producers
                .iter()
                .any(|identity| identity == role.producer())
        {
            return Err(denial(
                DenialKind::ContributionMemberMismatch,
                role.readiness(),
            ));
        }
        Ok(())
    }
}
