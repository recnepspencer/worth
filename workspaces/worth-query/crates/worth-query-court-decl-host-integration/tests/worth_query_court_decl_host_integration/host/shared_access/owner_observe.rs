//! Read-only owner observation queries for strictly passive Directory Home inspection.

use std::time::{Duration, Instant};

use worth_query_decl::facade::{
    application_query::*,
    application_schema::{fragment::Registration, *},
    worth_query_application_query, worth_query_portable_type, worth_query_query_binding,
    worth_query_structured_value_binding,
};
use worth_query_host::facade::{
    admission::authenticated_principal::{
        WorthQueryAdmissionClock, WorthQueryCancellationSource, WorthQueryRequestScope,
    },
    application_entry::WorthQueryApplicationRequestExt,
    primary_graph::{
        WorthQueryApplicationProjection, WorthQueryApplicationProjectionDenial,
        WorthQueryApplicationProjectionRow,
    },
    shared_access::ScopeGrantId,
    WorthQueryApplicationHome,
};

use super::capabilities::SharedPrincipalBinding;
use super::dst_oracle::{GrantState, SCOPES};
use super::mutation_handlers::SharedProjectProgram;
use super::mutations::U64ResultBinding;
use super::program::authenticate_local_owner;
use super::schema::{
    ExternalMapping, GrantActiveField, GrantIdField, GrantModeField, Principal, Project,
    ProjectGrant, ProjectKeyField, ProjectVersionField, SharedProjectSchema,
};

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct OwnerObserveParams;
worth_query_portable_type!(
    OwnerObserveParams => "worth.query.court.shared_access.owner_observe_params.v1"
);
worth_query_structured_value_binding!(pub OwnerObserveParamsBinding for OwnerObserveParams {
    identity: "worth.query.court.shared_access.owner_observe_params_binding.v1"
});

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ObservedGrantRow {
    pub active: bool,
    pub mode: String,
}
worth_query_portable_type!(
    ObservedGrantRow => "worth.query.court.shared_access.observed_grant_row.v1"
);
worth_query_structured_value_binding!(pub ObservedGrantRowBinding for ObservedGrantRow {
    identity: "worth.query.court.shared_access.observed_grant_row_binding.v1"
});

pub struct OwnerProjectVersionSlot;
pub struct OwnerGrantActiveSlot;
pub struct OwnerGrantModeSlot;
worth_query_portable_type!(
    OwnerProjectVersionSlot => "worth.query.court.shared_access.owner_project_ver_slot.v1"
);
worth_query_portable_type!(
    OwnerGrantActiveSlot => "worth.query.court.shared_access.owner_grant_active_slot.v1"
);
worth_query_portable_type!(
    OwnerGrantModeSlot => "worth.query.court.shared_access.owner_grant_mode_slot.v1"
);

// --- 1. OwnerProjectQuery ---
worth_query_application_query!(
    pub OwnerProjectQuery for SharedProjectSchema,
    identity "SharedOwnerProjectQuery",
    parameters OwnerObserveParamsBinding,
    result U64ResultBinding,
    scope Project => "Project",
    name "shared_owner_project_query"
);

fn owner_project_version_field() -> ApplicationQueryResultFieldRef<
    OwnerProjectQuery,
    OwnerProjectVersionSlot,
    SharedProjectSchema,
    ProjectVersionField,
> {
    ApplicationQueryResultFieldRef::new("version", ProjectVersionField::at("ProjectVersion"))
}

impl WorthQueryApplicationProjection<SharedProjectSchema, OwnerProjectQuery> for u64 {
    fn project(
        row: &WorthQueryApplicationProjectionRow<'_, SharedProjectSchema, OwnerProjectQuery>,
    ) -> Result<Self, WorthQueryApplicationProjectionDenial> {
        row.field(owner_project_version_field())
    }
}

#[derive(Clone, Debug)]
pub struct OwnerProjectRead(pub String);
impl OwnerProjectRead {
    fn into_key(self) -> String {
        self.0
    }
}
worth_query_structured_value_binding!(pub OwnerProjectReadInputBinding for OwnerProjectRead {
    identity: "worth.query.court.shared_access.owner_project_read_input.v1"
});
worth_query_query_binding!(
    pub OwnerProjectReadBinding for OwnerProjectRead,
    schema SharedProjectSchema,
    identity "worth.query.court.shared_access.owner_project_read_binding.v1",
    input OwnerProjectReadInputBinding,
    query OwnerProjectQuery,
    parameters OwnerObserveParamsBinding => |_| ApplicationQueryParameterSet::new(),
    result U64ResultBinding,
    principal SharedPrincipalBinding,
    mapping ExternalMapping,
    principal_entity Principal,
    principal_identity u64,
    identity_binding U64ApplicationValueBinding,
    scope ProjectKeyField,
    field ProjectKeyField::at("ProjectKey"),
    value OwnerProjectRead::into_key,
    limits results 1
);

impl Registration<SharedProjectSchema> for OwnerProjectQuery {
    fn register(
        builder: ApplicationSchemaDeclarationBuilder<SharedProjectSchema>,
    ) -> Result<
        ApplicationSchemaDeclarationBuilder<SharedProjectSchema>,
        ApplicationSchemaDeclarationDenial,
    > {
        let shape = ApplicationQueryResultShapeBuilder::<
            SharedProjectSchema,
            OwnerProjectQuery,
            Project,
            u64,
            U64ResultBinding,
        >::new(Project::reference())
        .field(owner_project_version_field())
        .build();
        let def = ApplicationQueryDefinitionBuilder::declare(Self::reference())
            .root(Project::reference())
            .scope(Project::reference())
            .result_shape(shape)
            .cardinality(ApplicationQueryCardinality::ExactlyOne)
            .disclosure(ApplicationQueryDisclosureContract::public())
            .basis_support(ApplicationQueryBasisSupport::current_and_pinned())
            .lanes(ApplicationQueryLaneEligibility::one_shot())
            .public();
        Ok(builder.application_query(def))
    }
}

impl Registration<SharedProjectSchema> for OwnerProjectReadBinding {
    fn register(
        builder: ApplicationSchemaDeclarationBuilder<SharedProjectSchema>,
    ) -> Result<
        ApplicationSchemaDeclarationBuilder<SharedProjectSchema>,
        ApplicationSchemaDeclarationDenial,
    > {
        Ok(builder.application_query_binding::<Self>())
    }
}

// --- 2. OwnerGrantQuery ---
worth_query_application_query!(
    pub OwnerGrantQuery for SharedProjectSchema,
    identity "SharedOwnerGrantQuery",
    parameters OwnerObserveParamsBinding,
    result ObservedGrantRowBinding,
    scope ProjectGrant => "ProjectGrant",
    name "shared_owner_grant_query"
);

fn owner_grant_active_field() -> ApplicationQueryResultFieldRef<
    OwnerGrantQuery,
    OwnerGrantActiveSlot,
    SharedProjectSchema,
    GrantActiveField,
> {
    ApplicationQueryResultFieldRef::new("active", GrantActiveField::at("GrantActive"))
}

fn owner_grant_mode_field() -> ApplicationQueryResultFieldRef<
    OwnerGrantQuery,
    OwnerGrantModeSlot,
    SharedProjectSchema,
    GrantModeField,
> {
    ApplicationQueryResultFieldRef::new("mode", GrantModeField::at("GrantMode"))
}

impl WorthQueryApplicationProjection<SharedProjectSchema, OwnerGrantQuery> for ObservedGrantRow {
    fn project(
        row: &WorthQueryApplicationProjectionRow<'_, SharedProjectSchema, OwnerGrantQuery>,
    ) -> Result<Self, WorthQueryApplicationProjectionDenial> {
        Ok(Self {
            active: row.field(owner_grant_active_field())?,
            mode: row.field(owner_grant_mode_field())?,
        })
    }
}

#[derive(Clone, Debug)]
pub struct OwnerGrantRead(pub String);
impl OwnerGrantRead {
    fn into_key(self) -> String {
        self.0
    }
}
worth_query_structured_value_binding!(pub OwnerGrantReadInputBinding for OwnerGrantRead {
    identity: "worth.query.court.shared_access.owner_grant_read_input.v1"
});
worth_query_query_binding!(
    pub OwnerGrantReadBinding for OwnerGrantRead,
    schema SharedProjectSchema,
    identity "worth.query.court.shared_access.owner_grant_read_binding.v1",
    input OwnerGrantReadInputBinding,
    query OwnerGrantQuery,
    parameters OwnerObserveParamsBinding => |_| ApplicationQueryParameterSet::new(),
    result ObservedGrantRowBinding,
    principal SharedPrincipalBinding,
    mapping ExternalMapping,
    principal_entity Principal,
    principal_identity u64,
    identity_binding U64ApplicationValueBinding,
    scope GrantIdField,
    field GrantIdField::at("GrantId"),
    value OwnerGrantRead::into_key,
    limits results 1
);

impl Registration<SharedProjectSchema> for OwnerGrantQuery {
    fn register(
        builder: ApplicationSchemaDeclarationBuilder<SharedProjectSchema>,
    ) -> Result<
        ApplicationSchemaDeclarationBuilder<SharedProjectSchema>,
        ApplicationSchemaDeclarationDenial,
    > {
        let shape = ApplicationQueryResultShapeBuilder::<
            SharedProjectSchema,
            OwnerGrantQuery,
            ProjectGrant,
            ObservedGrantRow,
            ObservedGrantRowBinding,
        >::new(ProjectGrant::reference())
        .field(owner_grant_active_field())
        .field(owner_grant_mode_field())
        .build();
        let def = ApplicationQueryDefinitionBuilder::declare(Self::reference())
            .root(ProjectGrant::reference())
            .scope(ProjectGrant::reference())
            .result_shape(shape)
            .cardinality(ApplicationQueryCardinality::ExactlyOne)
            .disclosure(ApplicationQueryDisclosureContract::public())
            .basis_support(ApplicationQueryBasisSupport::current_and_pinned())
            .lanes(ApplicationQueryLaneEligibility::one_shot())
            .public();
        Ok(builder.application_query(def))
    }
}

impl Registration<SharedProjectSchema> for OwnerGrantReadBinding {
    fn register(
        builder: ApplicationSchemaDeclarationBuilder<SharedProjectSchema>,
    ) -> Result<
        ApplicationSchemaDeclarationBuilder<SharedProjectSchema>,
        ApplicationSchemaDeclarationDenial,
    > {
        Ok(builder.application_query_binding::<Self>())
    }
}

/// Passively reads committed project versions and durable grant states straight from the Home.
pub fn observe_durable_home_state(
    home: &WorthQueryApplicationHome<SharedProjectSchema, SharedProjectProgram>,
    grant_ids: &[[Option<ScopeGrantId>; 2]; 2],
) -> Result<([u64; 2], [[GrantState; 2]; 2]), String> {
    let cancel = WorthQueryCancellationSource::new();
    let scope = WorthQueryRequestScope::new(
        Some(Instant::now() + Duration::from_secs(15)),
        cancel.token(),
        WorthQueryAdmissionClock::system(),
    );
    let app = home.runtime().runtime();
    let owner = authenticate_local_owner(app.installed_schema(), &scope)
        .map_err(|e| format!("owner observation auth failed: {e:?}"))?;
    let mut values = [0_u64; 2];
    for (s, key) in SCOPES.iter().enumerate() {
        let published = app
            .request(&owner, &scope)
            .query(OwnerProjectRead((*key).to_owned()))
            .execute()
            .map_err(|e| format!("owner project observation failed for {key}: {e:?}"))?;
        values[s] = *published
            .rows()
            .first()
            .ok_or_else(|| format!("missing Project row for {key}"))?;
    }
    let mut grants = [[GrantState::None; 2]; 2];
    for p in 0..2 {
        for s in 0..2 {
            let Some(grant_id) = grant_ids[p][s].as_ref() else {
                continue;
            };
            let published = app
                .request(&owner, &scope)
                .query(OwnerGrantRead(grant_id.as_str().to_owned()))
                .execute()
                .map_err(|e| format!("owner grant observation failed for {grant_id}: {e:?}"))?;
            let row = published
                .rows()
                .first()
                .ok_or_else(|| format!("missing ProjectGrant row for {grant_id}"))?;
            grants[p][s] = if !row.active {
                GrantState::Revoked
            } else if row.mode == "read_and_watch" {
                GrantState::ActiveReadAndWatch
            } else {
                GrantState::ActiveReadOnly
            };
        }
    }
    Ok((values, grants))
}
