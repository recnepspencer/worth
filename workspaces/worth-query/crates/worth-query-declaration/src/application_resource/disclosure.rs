use std::marker::PhantomData;

use crate::application_query::{
    ApplicationQueryObservableInfluence, ApplicationQueryResultSlotKey,
};
use crate::portable_identity::{WorthQueryPortableType, WorthQueryPortableTypeIdentity};

#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub enum ApplicationResourceDisclosureFacet {
    Availability,
    State,
    Result,
    DomainFailure,
    CleanupDetail,
}

impl ApplicationResourceDisclosureFacet {
    pub const ALL: [Self; 5] = [
        Self::Availability,
        Self::State,
        Self::Result,
        Self::DomainFailure,
        Self::CleanupDetail,
    ];

    pub const fn name(self) -> &'static str {
        match self {
            Self::Availability => "resource-availability",
            Self::State => "resource-state",
            Self::Result => "resource-result",
            Self::DomainFailure => "resource-domain-failure",
            Self::CleanupDetail => "resource-cleanup-detail",
        }
    }

    /// Availability is boolean existence and State is lifecycle classification.
    /// Queue/provider counts may appear only in separately admitted cleanup detail.
    pub const fn required_influences(self) -> &'static [ApplicationQueryObservableInfluence] {
        match self {
            Self::Availability => &[ApplicationQueryObservableInfluence::RowPresence],
            Self::State => &[ApplicationQueryObservableInfluence::LiveMembership],
            Self::CleanupDetail => &[
                ApplicationQueryObservableInfluence::Explanation,
                ApplicationQueryObservableInfluence::Count,
            ],
            Self::Result | Self::DomainFailure => &[],
        }
    }
}

#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub struct ApplicationResourceDisclosureMeaning {
    resource: WorthQueryPortableTypeIdentity,
    facet: ApplicationResourceDisclosureFacet,
}

impl ApplicationResourceDisclosureMeaning {
    pub fn from_untrusted_fields(
        resource: WorthQueryPortableTypeIdentity,
        facet: ApplicationResourceDisclosureFacet,
    ) -> Self {
        Self { resource, facet }
    }

    pub fn resource_identity(&self) -> WorthQueryPortableTypeIdentity {
        self.resource.clone()
    }
    pub const fn resource_type(&self) -> &str {
        self.resource.as_str()
    }
    pub const fn facet(&self) -> ApplicationResourceDisclosureFacet {
        self.facet
    }
    pub const fn slot_type(&self) -> &'static str {
        self.facet.name()
    }
    pub fn slot_key(&self) -> ApplicationQueryResultSlotKey {
        ApplicationQueryResultSlotKey::resource(self.resource.clone(), self.facet)
    }
}

pub struct ApplicationResourceDisclosureRef<Resource> {
    facet: ApplicationResourceDisclosureFacet,
    _resource: PhantomData<fn() -> Resource>,
}

impl<Resource: WorthQueryPortableType> ApplicationResourceDisclosureRef<Resource> {
    pub const fn new(facet: ApplicationResourceDisclosureFacet) -> Self {
        Self {
            facet,
            _resource: PhantomData,
        }
    }

    pub fn meaning(&self) -> ApplicationResourceDisclosureMeaning {
        ApplicationResourceDisclosureMeaning::from_untrusted_fields(
            Resource::PORTABLE_TYPE_IDENTITY,
            self.facet,
        )
    }
}
