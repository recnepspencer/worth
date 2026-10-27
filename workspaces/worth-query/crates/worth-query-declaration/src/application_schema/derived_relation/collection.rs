//! Descriptive association with the existing managed collection inventory.
use crate::application_program::{
    ApplicationCollectionCompleteness, ApplicationDerivedArtifact, ApplicationDerivedCollection,
    ApplicationFeature, ApplicationManagedComputation,
};
use crate::application_schema::ApplicationSchema;

/// Existing declared identities, not a portable Rust type name or value codec.
/// Installation must compare these with the actual typed setup owner.
#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub struct ApplicationRuleReadableCollectionAssociation {
    pub relation: String,
    pub feature: String,
    pub feature_major: u16,
    pub feature_minor: u16,
    pub collection: String,
    pub computation: String,
    pub output_artifact: String,
    pub completeness: String,
}

impl ApplicationRuleReadableCollectionAssociation {
    /// Checks names before completed-schema and actual setup-owner matching.
    pub fn validate(&self) -> Result<(), super::ApplicationDerivedRelationDefinitionDenial> {
        use super::ApplicationDerivedRelationDefinitionDenial as Denial;
        super::super::identifier_validation::validate_simple_identifier(&self.relation)
            .map_err(|_| Denial::InvalidIdentifier)?;
        if [
            &self.feature,
            &self.collection,
            &self.computation,
            &self.output_artifact,
            &self.completeness,
        ]
        .into_iter()
        .any(|identity| identity.is_empty() || identity.trim() != identity.as_str())
        {
            return Err(Denial::InvalidCollectionAssociation);
        }
        Ok(())
    }

    /// Projects the actual existing trait identities. The result remains
    /// descriptive; only a configured framework owner can bind this bridge.
    pub fn of<Schema, Feature, Collection>(relation: &str) -> Self
    where
        Schema: ApplicationSchema,
        Feature: ApplicationFeature<Schema>,
        Collection: ApplicationDerivedCollection<Schema, Feature>,
    {
        Self {
            relation: relation.to_owned(),
            feature: Feature::IDENTITY.to_owned(),
            feature_major: Feature::MAJOR,
            feature_minor: Feature::MINOR,
            collection: Collection::IDENTITY.to_owned(),
            computation: Collection::Computation::IDENTITY.to_owned(),
            output_artifact: <<Collection::Computation as ApplicationManagedComputation<
                Schema,
                Feature,
            >>::Output as ApplicationDerivedArtifact<Schema, Feature>>::IDENTITY
                .to_owned(),
            completeness: Collection::Completeness::IDENTITY.to_owned(),
        }
    }
}
