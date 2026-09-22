use std::any::TypeId;

use crate::application_schema::ApplicationSchema;

use super::ApplicationFeature;

pub trait ApplicationCollectionContributor: 'static {
    const IDENTITY: &'static str;
}

pub trait ApplicationCollectionGrouping: 'static {
    const IDENTITY: &'static str;
}

pub trait ApplicationCollectionMeasures: 'static {
    const IDENTITY: &'static str;
}

pub trait ApplicationCollectionLineage: 'static {
    const IDENTITY: &'static str;
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ApplicationCollectionIncompletePosture {
    ExcludeIncomplete,
    RetainPending,
}

impl ApplicationCollectionIncompletePosture {
    /// Names this posture in the durable canonical manifest record that the
    /// program revision digests.
    ///
    /// The token is decided here rather than derived from the Rust variant
    /// spelling, so renaming a variant is a compile-time-visible decision
    /// instead of a silent change of every program's content identity.
    pub const fn canonical_token(self) -> &'static str {
        match self {
            Self::ExcludeIncomplete => "ExcludeIncomplete",
            Self::RetainPending => "RetainPending",
        }
    }
}

pub trait ApplicationDerivedCollection<Schema, Feature>: Sized + 'static
where
    Schema: ApplicationSchema,
    Feature: ApplicationFeature<Schema>,
{
    type Contributor: ApplicationCollectionContributor;
    type Grouping: ApplicationCollectionGrouping;
    type Measures: ApplicationCollectionMeasures;
    type Lineage: ApplicationCollectionLineage;

    const IDENTITY: &'static str;
    const APPLICABILITY: &'static str;
    const INCOMPLETE: ApplicationCollectionIncompletePosture;
    const INCREMENTAL_UPDATE: &'static str;
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ApplicationDerivedCollectionDeclaration {
    identity: &'static str,
    collection_type: TypeId,
    feature_type: TypeId,
    contributor: &'static str,
    contributor_type: TypeId,
    grouping: &'static str,
    measures: &'static str,
    lineage: &'static str,
    applicability: &'static str,
    incomplete: ApplicationCollectionIncompletePosture,
    incremental_update: &'static str,
}

impl ApplicationDerivedCollectionDeclaration {
    pub(crate) fn of<Schema, Feature, Collection>() -> Self
    where
        Schema: ApplicationSchema,
        Feature: ApplicationFeature<Schema>,
        Collection: ApplicationDerivedCollection<Schema, Feature>,
    {
        Self {
            identity: Collection::IDENTITY,
            collection_type: TypeId::of::<Collection>(),
            feature_type: TypeId::of::<Feature>(),
            contributor: Collection::Contributor::IDENTITY,
            contributor_type: TypeId::of::<Collection::Contributor>(),
            grouping: Collection::Grouping::IDENTITY,
            measures: Collection::Measures::IDENTITY,
            lineage: Collection::Lineage::IDENTITY,
            applicability: Collection::APPLICABILITY,
            incomplete: Collection::INCOMPLETE,
            incremental_update: Collection::INCREMENTAL_UPDATE,
        }
    }

    pub const fn identity(&self) -> &'static str {
        self.identity
    }
    pub const fn collection_type(&self) -> TypeId {
        self.collection_type
    }
    pub const fn feature_type(&self) -> TypeId {
        self.feature_type
    }
    pub const fn contributor(&self) -> &'static str {
        self.contributor
    }
    pub const fn contributor_type(&self) -> TypeId {
        self.contributor_type
    }
    pub const fn grouping(&self) -> &'static str {
        self.grouping
    }
    pub const fn measures(&self) -> &'static str {
        self.measures
    }
    pub const fn lineage(&self) -> &'static str {
        self.lineage
    }
    pub const fn applicability(&self) -> &'static str {
        self.applicability
    }
    pub const fn incomplete(&self) -> ApplicationCollectionIncompletePosture {
        self.incomplete
    }
    pub const fn incremental_update(&self) -> &'static str {
        self.incremental_update
    }
}
