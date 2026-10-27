//! Only the actual closed owner can satisfy a rule-readable count association.
use super::super::{
    WorthQueryDerivedRelationCountOwner, WorthQueryInstalledPartitionedComputation,
};
use super::*;
use crate::domain_computation::primary_graph::WorthQueryInvariantEntityIdentity;
use worth_query_declaration::facade::{
    application_program::*,
    application_schema::{
        ApplicationDerivedRelationRef, ApplicationOperationMarkerIdentity,
        ApplicationRuleReadableCollectionAssociation, OperationReads,
    },
};

impl<Schema: ApplicationSchema> WorthQueryApplicationContributionSetup<'_, Schema> {
    /// Installs a count of one recipe's canonical endpoint pairs using the
    /// existing collection owner, original input digest and retained read facts.
    pub fn derived_relation_count<Feature, Collection, Operation, Relation, From, To>(
        &mut self,
        relation: ApplicationDerivedRelationRef<Schema, Relation, From, To>,
    ) -> Result<
        WorthQueryInstalledPartitionedComputation<
            Schema,
            Feature,
            Collection::Computation,
            WorthQueryDerivedRelationCountOwner<
                Schema,
                Feature,
                Collection,
                Operation,
                Relation,
                From,
                To,
            >,
        >,
        WorthQueryPrimaryGraphInstallationDenial,
    >
    where
        Feature: ApplicationFeature<Schema>,
        Collection: ApplicationDerivedCollection<Schema, Feature>,
        Collection::Computation: ApplicationManagedComputation<
            Schema,
            Feature,
            Partition = ApplicationDerivedRelationCountPartition,
        >,
        <Collection::Computation as ApplicationManagedComputation<Schema, Feature>>::Input:
            ApplicationComputationInput<Value = WorthQueryInvariantEntityIdentity<Schema, From>>,
        Collection::Completeness: ApplicationCollectionCompleteness<Value = u64>,
        <Collection::Completeness as ApplicationCollectionCompleteness>::Stopped: Send,
        Operation: ApplicationOperationMarkerIdentity<Schema> + 'static,
        Relation: OperationReads<Operation> + 'static,
        From: 'static,
        To: 'static,
    {
        let expected =
            ApplicationRuleReadableCollectionAssociation::of::<Schema, Feature, Collection>(
                relation.name(),
            );
        let members = self.installed_schema.installed_declaration().members();
        let compatible=members.iter().any(|member|matches!(member,
            ApplicationSchemaMember::DerivedRelation { definition } if definition.parts().relation==relation.name()
                && definition.parts().from.borrowed()==relation.from_member() && definition.parts().to.borrowed()==relation.to_member()
                && definition.parts().from.entity().is_some()))
            && members.iter().any(|member|matches!(member,
                ApplicationSchemaMember::OperationDerivedRead { operation,relation:declared }
                    if operation==Operation::IDENTIFIER && declared==relation.name()));
        if !compatible || !self.contribution.members().any(|member| matches!(member,
            ApplicationSchemaMember::RuleReadableCollection { association } if association==&expected)) {
            return Err(WorthQueryPrimaryGraphInstallationDenial::new(
                WorthQueryPrimaryGraphInstallationDenialKind::CollectionOwnerMeaningMismatch,relation.name()));
        }
        let installed = self.derived_collection::<Feature, Collection, _>(
            WorthQueryDerivedRelationCountOwner::<
                Schema,
                Feature,
                Collection,
                Operation,
                Relation,
                From,
                To,
            >::new(relation, expected),
        )?;
        self.handlers
            .bind_rule_readable_count::<Feature, Collection>(relation.name());
        Ok(installed)
    }
}
