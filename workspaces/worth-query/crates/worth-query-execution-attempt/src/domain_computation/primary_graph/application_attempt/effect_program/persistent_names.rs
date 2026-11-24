//! Persistent-name births and lifecycle postures over a program's outputs.
//!
//! A program declares what each created output means and which held names
//! its deletions retire, split or merge. Nothing here mints, reads or
//! forges a name: the declarations ride the program to its commit, where
//! Relational validates them against the performed effects and mints.
//! Unnamed programs carry an empty declaration and attach no naming plan.

use worth_query_declaration::facade::application_naming::{
    ApplicationGroupKind, ApplicationNameSubjectKind as Subject, WorthQueryGroupPartition,
    WorthQueryName,
};
use worth_query_execution_lineage::facade::ports::CreatedRelationInternalPort as _;
use worth_query_execution_lineage::facade::persistent_naming::{
    admit_output_birth, admit_same_commit_names, OperationMergeKeeper, OperationNamedOutput,
    OperationNamingEntry, OperationOutputKey, OutputBirthPosture as Posture, WorthQueryOutputBirth,
    WorthQueryOutputMergeKeeper, WorthQueryOutputNameDenial,
};

use super::{
    EffectEntityBorrowedExt, WorthQueryApplicationAttemptDenial,
    WorthQueryApplicationCreatedRelation, WorthQueryApplicationEffectEntity,
    WorthQueryApplicationEffectProgramBuilder, WorthQueryApplicationRealizedEffect,
};
use worth_relational::facade::transactions::EntityReference;

type Refused = WorthQueryApplicationAttemptDenial;

pub(super) fn refused(denial: WorthQueryOutputNameDenial) -> Refused {
    WorthQueryApplicationAttemptDenial::persistent_name_output(denial)
}

impl<Schema, Operation, Input, Scope>
    WorthQueryApplicationEffectProgramBuilder<Schema, Operation, Input, Scope>
{
    /// Declares the birth of `created`, an entity this program created, under
    /// its kind's installed scheme. Re-minting a name whose holder this
    /// program retires (`retire_entity_name`) records `Preserve` continuity.
    pub fn name_created_entity<Entity>(
        &mut self,
        created: &WorthQueryApplicationEffectEntity<Schema, Entity>,
        birth: WorthQueryOutputBirth,
    ) -> Result<(), Refused> {
        let output = self.entity_output(created, birth, Posture::Birth)?;
        self.persistent_names
            .push(OperationNamingEntry::Birth(output));
        Ok(())
    }

    /// Declares the birth of `created`, a relation this program linked with
    /// `link_for_initialization`. Parallel same-role relations between the
    /// same endpoints are told apart by their discriminators.
    pub fn name_created_relation<Relation, From, To>(
        &mut self,
        created: &WorthQueryApplicationCreatedRelation<Schema, Relation, From, To>,
        birth: WorthQueryOutputBirth,
    ) -> Result<(), Refused> {
        let output = self.relation_output(created, birth, Posture::Birth)?;
        self.persistent_names
            .push(OperationNamingEntry::Birth(output));
        Ok(())
    }

    /// Ends `held` with the entity this program deletes. A plain deletion
    /// retires its name too; declaring it lets this program re-mint the same
    /// name in place (`Preserve`).
    pub fn retire_entity_name<Entity>(
        &mut self,
        held: WorthQueryName<Entity>,
    ) -> Result<(), Refused> {
        self.push_retire(Subject::Entity, *held.as_bytes())
    }

    /// Ends `held` with the relation this program unlinks.
    pub fn retire_relation_name<Relation>(
        &mut self,
        held: WorthQueryName<Relation>,
    ) -> Result<(), Refused> {
        self.push_retire(Subject::Relation, *held.as_bytes())
    }

    /// Splits `parent`, whose entity this program deletes, into `children`
    /// this program created. Every child carries a discriminator.
    pub fn split_entity_name<Entity>(
        &mut self,
        parent: WorthQueryName<Entity>,
        children: Vec<(
            &WorthQueryApplicationEffectEntity<Schema, Entity>,
            WorthQueryOutputBirth,
        )>,
    ) -> Result<(), Refused> {
        let children = children
            .into_iter()
            .map(|(child, birth)| self.entity_output(child, birth, Posture::SplitChild))
            .collect::<Result<_, _>>()?;
        self.persistent_names.push(OperationNamingEntry::Split {
            subject: Subject::Entity,
            parent: *parent.as_bytes(),
            children,
        });
        Ok(())
    }

    /// Splits `parent`, whose relation this program unlinks, into created
    /// relation `children`.
    #[allow(clippy::type_complexity)]
    pub fn split_relation_name<Relation, From, To>(
        &mut self,
        parent: WorthQueryName<Relation>,
        children: Vec<(
            &WorthQueryApplicationCreatedRelation<Schema, Relation, From, To>,
            WorthQueryOutputBirth,
        )>,
    ) -> Result<(), Refused> {
        let children = children
            .into_iter()
            .map(|(child, birth)| self.relation_output(child, birth, Posture::SplitChild))
            .collect::<Result<_, _>>()?;
        self.persistent_names.push(OperationNamingEntry::Split {
            subject: Subject::Relation,
            parent: *parent.as_bytes(),
            children,
        });
        Ok(())
    }

    /// Merges held entity names under `keeper`. Every parent except a kept
    /// in-place keeper must be deleted by this program.
    pub fn merge_entity_names<Entity>(
        &mut self,
        parents: &[WorthQueryName<Entity>],
        keeper: WorthQueryOutputMergeKeeper<
            Entity,
            &WorthQueryApplicationEffectEntity<Schema, Entity>,
        >,
    ) -> Result<(), Refused> {
        let keeper = match keeper {
            WorthQueryOutputMergeKeeper::KeepParentInPlace(name) => {
                OperationMergeKeeper::KeepParentInPlace(*name.as_bytes())
            }
            WorthQueryOutputMergeKeeper::KeepParentOnto { keeper, target } => {
                OperationMergeKeeper::KeepParentOnto {
                    keeper: *keeper.as_bytes(),
                    target: self.created_entity_key(target)?.0,
                }
            }
            WorthQueryOutputMergeKeeper::MintNew { target, birth } => {
                OperationMergeKeeper::MintNew(self.entity_output(
                    target,
                    birth,
                    Posture::MergeSurvivor,
                )?)
            }
        };
        self.push_merge(Subject::Entity, parents, keeper)
    }

    /// Merges held relation names under `keeper`.
    pub fn merge_relation_names<Relation, From, To>(
        &mut self,
        parents: &[WorthQueryName<Relation>],
        keeper: WorthQueryOutputMergeKeeper<
            Relation,
            &WorthQueryApplicationCreatedRelation<Schema, Relation, From, To>,
        >,
    ) -> Result<(), Refused> {
        let keeper = match keeper {
            WorthQueryOutputMergeKeeper::KeepParentInPlace(name) => {
                OperationMergeKeeper::KeepParentInPlace(*name.as_bytes())
            }
            WorthQueryOutputMergeKeeper::KeepParentOnto { keeper, target } => {
                OperationMergeKeeper::KeepParentOnto {
                    keeper: *keeper.as_bytes(),
                    target: self.created_relation_key(target)?.0,
                }
            }
            WorthQueryOutputMergeKeeper::MintNew { target, birth } => {
                OperationMergeKeeper::MintNew(self.relation_output(
                    target,
                    birth,
                    Posture::MergeSurvivor,
                )?)
            }
        };
        self.push_merge(Subject::Relation, parents, keeper)
    }

    fn persistent_name_catalog(
        &self,
    ) -> Option<&worth_query_execution_lineage::facade::persistent_naming::LoweredExecutionNameSchemeCatalog> {
        self.layout.persistent_names().map(|names| names.catalog())
    }

    fn admit_output(
        &self,
        subject: Subject,
        subject_name: String,
        target: OperationOutputKey,
        birth: WorthQueryOutputBirth,
        posture: Posture,
    ) -> Result<OperationNamedOutput<OperationOutputKey>, Refused> {
        let catalog = self.persistent_name_catalog().ok_or_else(|| {
            refused(WorthQueryOutputNameDenial::UnschemedOutput {
                subject,
                subject_name: subject_name.clone(),
            })
        })?;
        let scheme_id = admit_output_birth(catalog, subject, &subject_name, &birth, posture)
            .map_err(refused)?;
        admit_same_commit_names(catalog, &self.persistent_names, &scheme_id, &birth)
            .map_err(refused)?;
        Ok(OperationNamedOutput {
            target,
            scheme_id,
            birth,
        })
    }

    fn entity_output<Entity>(
        &self,
        created: &WorthQueryApplicationEffectEntity<Schema, Entity>,
        birth: WorthQueryOutputBirth,
        posture: Posture,
    ) -> Result<OperationNamedOutput<OperationOutputKey>, Refused> {
        let (target, entity) = self.created_entity_key(created)?;
        self.admit_output(Subject::Entity, entity, target, birth, posture)
    }

    fn relation_output<Relation, From, To>(
        &self,
        created: &WorthQueryApplicationCreatedRelation<Schema, Relation, From, To>,
        birth: WorthQueryOutputBirth,
        posture: Posture,
    ) -> Result<OperationNamedOutput<OperationOutputKey>, Refused> {
        let (target, relation) = self.created_relation_key(created)?;
        self.admit_output(Subject::Relation, relation, target, birth, posture)
    }

    /// The stable key of an entity this program created; existing and
    /// foreign targets refuse.
    pub(super) fn created_entity_key<Entity>(
        &self,
        created: &WorthQueryApplicationEffectEntity<Schema, Entity>,
    ) -> Result<(OperationOutputKey, String), Refused> {
        let target = created.borrowed();
        let not_created = || {
            refused(WorthQueryOutputNameDenial::NotCreatedOutput {
                subject: Subject::Entity,
                subject_name: target.entity.to_owned(),
            })
        };
        if !std::sync::Arc::ptr_eq(target.program, &self.program) || target.created_effect.is_none()
        {
            return Err(not_created());
        }
        let EntityReference::Created(reference) = target.reference else {
            return Err(not_created());
        };
        let key = reference.client_key.as_raw_str().ok_or_else(not_created)?;
        Ok((
            OperationOutputKey {
                subject: Subject::Entity,
                kind: reference.kind_id,
                key: key.to_owned(),
            },
            target.entity.to_owned(),
        ))
    }

    /// The stable key of a relation this program linked.
    pub(super) fn created_relation_key<Relation, From, To>(
        &self,
        created: &WorthQueryApplicationCreatedRelation<Schema, Relation, From, To>,
    ) -> Result<(OperationOutputKey, String), Refused> {
        let name = created.relation().name().to_owned();
        let not_created = || {
            refused(WorthQueryOutputNameDenial::NotCreatedOutput {
                subject: Subject::Relation,
                subject_name: name.clone(),
            })
        };
        if !std::sync::Arc::ptr_eq(created.program(), &self.program) {
            return Err(not_created());
        }
        let Some(WorthQueryApplicationRealizedEffect::CreateRelation { kind, key, .. }) =
            self.effects.get(created.created_effect())
        else {
            return Err(not_created());
        };
        Ok((
            OperationOutputKey {
                subject: Subject::Relation,
                kind: *kind,
                key: key.clone(),
            },
            name,
        ))
    }

    /// Declares group kind `G`'s next partition of its changed scope. Group
    /// names are never minted here: Relational names each declared group by
    /// `G`'s continuity rule when the commit finalizes, and an empty
    /// partition declares nothing.
    pub fn declare_group_partition<G: ApplicationGroupKind>(
        &mut self,
        partition: WorthQueryGroupPartition<G>,
    ) -> Result<(), Refused> {
        if partition.is_empty() {
            return Ok(());
        }
        let (groups, dissolved) = partition.into_parts();
        let groups = groups
            .into_iter()
            .map(|members| {
                let members = members.into_iter();
                members.map(|(role, name)| (role.to_owned(), name)).collect()
            })
            .collect();
        self.persistent_names
            .push(OperationNamingEntry::GroupPartition {
                group_name: G::NAME.to_owned(),
                dissolved,
                groups,
            });
        Ok(())
    }

    fn push_retire(&mut self, subject: Subject, name: [u8; 32]) -> Result<(), Refused> {
        self.persistent_names
            .push(OperationNamingEntry::Retire { subject, name });
        Ok(())
    }

    fn push_merge<K>(
        &mut self,
        subject: Subject,
        parents: &[WorthQueryName<K>],
        keeper: OperationMergeKeeper<OperationOutputKey>,
    ) -> Result<(), Refused> {
        self.persistent_names.push(OperationNamingEntry::Merge {
            subject,
            parents: parents.iter().map(|parent| *parent.as_bytes()).collect(),
            keeper,
        });
        Ok(())
    }
}
