//! Collection membership in the existing installed computation inventory.
use super::*;
use worth_query_declaration::facade::application_program::{
    ApplicationCollectionCompleteness, ApplicationDerivedCollection, ApplicationFeature,
    ApplicationFeatureDeclaration,
};

#[derive(Clone, Copy)]
pub(super) struct InstalledCollection {
    collection_type: TypeId,
    completeness_type: TypeId,
    value_type: TypeId,
}

impl<Schema: ApplicationSchema> PendingMutationHandlerRegistry<Schema> {
    pub(in crate::domain_computation::primary_graph) fn bind_collection<Feature, Collection>(
        &mut self,
    ) where
        Feature: ApplicationFeature<Schema>,
        Collection: ApplicationDerivedCollection<Schema, Feature>,
    {
        self.computations
            .get_mut(&TypeId::of::<Collection::Computation>())
            .expect("collection installation just recorded its computation")
            .collection = Some(InstalledCollection {
            collection_type: TypeId::of::<Collection>(),
            completeness_type: TypeId::of::<Collection::Completeness>(),
            value_type: TypeId::of::<
                <Collection::Completeness as ApplicationCollectionCompleteness>::Value,
            >(),
        });
    }
}

impl<Schema> PendingMutationHandlerRegistry<Schema> {
    pub(in crate::domain_computation::primary_graph) fn validate_derived_collections(
        &self,
        features: &[ApplicationFeatureDeclaration],
    ) -> Result<(), WorthQueryPrimaryGraphInstallationDenial> {
        let mut declared = features
            .iter()
            .flat_map(|feature| {
                feature
                    .derived_collections()
                    .iter()
                    .map(move |collection| (feature, collection))
            })
            .collect::<Vec<_>>();
        declared.sort_by_key(|(feature, collection)| {
            (
                feature.composition_instance(),
                feature.identity(),
                collection.identity(),
            )
        });
        let mut expected = std::collections::HashSet::new();
        for (_, collection) in declared {
            expected.insert(TypeId::from(collection.collection_type()));
            let installed = self
                .computations
                .get(&TypeId::from(collection.computation_type()))
                .and_then(|owner| owner.collection)
                .ok_or_else(|| denial(DenialKind::MissingCollectionOwner, collection.identity()))?;
            if installed.collection_type != TypeId::from(collection.collection_type())
                || installed.completeness_type != TypeId::from(collection.completeness_type())
                || installed.value_type != TypeId::from(collection.value_type())
            {
                return Err(denial(
                    DenialKind::CollectionOwnerMeaningMismatch,
                    collection.identity(),
                ));
            }
        }
        if features
            .iter()
            .flat_map(|feature| feature.managed_computations())
            .filter_map(|computation| {
                self.computations
                    .get(&TypeId::from(computation.computation_type()))
            })
            .filter_map(|owner| owner.collection)
            .any(|collection| !expected.contains(&collection.collection_type))
        {
            return Err(denial(
                DenialKind::ForeignCollectionOwner,
                "derived collection owner inventory",
            ));
        }
        Ok(())
    }
}
