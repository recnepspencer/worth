//! Uniform entity and relation lineage output-role descriptors, contracts, and
//! compile-time verified role/family declarations.

use std::marker::PhantomData;

use crate::application_schema::{fragment, ApplicationEntityMarkerIdentity, ApplicationSchema};

use super::super::output::ApplicationMutationOutputRoleCardinality;
use super::action::{
    ApplicationMutationLineagePostureKind, ApplicationMutationLineagePostureSet,
    WorthQueryApplicationLineageOutputAction,
};
use super::lineage_posture::ApplicationMutationLineageSubjectKind;
use super::{
    same_text, WorthQueryApplicationOutputCardinality, WorthQueryApplicationOutputRoleNameDenial,
};

mod sealed {
    pub trait Sealed {}
}

/// Sealed subject marker wrapper identifying an entity output subject.
pub struct WorthQueryEntityOutputSubject<E>(PhantomData<fn() -> E>);

/// Sealed subject marker wrapper identifying a relation output subject.
pub struct WorthQueryRelationOutputSubject<R>(PhantomData<fn() -> R>);

/// Uniform subject identity trait for entity and relation lineage output roles.
pub trait WorthQueryApplicationOutputSubject<Schema>: sealed::Sealed + 'static {
    const SUBJECT_KIND: ApplicationMutationLineageSubjectKind;
    const SUBJECT_IDENTIFIER: &'static str;
}

impl<E: 'static> sealed::Sealed for WorthQueryEntityOutputSubject<E> {}
impl<Schema, E> WorthQueryApplicationOutputSubject<Schema> for WorthQueryEntityOutputSubject<E>
where
    E: ApplicationEntityMarkerIdentity<Schema> + 'static,
{
    const SUBJECT_KIND: ApplicationMutationLineageSubjectKind =
        ApplicationMutationLineageSubjectKind::Entity;
    const SUBJECT_IDENTIFIER: &'static str = E::IDENTIFIER;
}

impl<R: 'static> sealed::Sealed for WorthQueryRelationOutputSubject<R> {}
impl<Schema, R> WorthQueryApplicationOutputSubject<Schema> for WorthQueryRelationOutputSubject<R>
where
    R: fragment::Relation,
{
    const SUBJECT_KIND: ApplicationMutationLineageSubjectKind =
        ApplicationMutationLineageSubjectKind::Relation;
    const SUBJECT_IDENTIFIER: &'static str = R::NAME;
}

/// Installed descriptor for one fixed entity or relation lineage output role.
#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub struct ApplicationMutationLineageOutputRoleDescriptor {
    name: &'static str,
    subject_kind: ApplicationMutationLineageSubjectKind,
    subject: &'static str,
    posture: ApplicationMutationLineagePostureKind,
    cardinality: ApplicationMutationOutputRoleCardinality,
}

impl ApplicationMutationLineageOutputRoleDescriptor {
    pub(super) const fn declared(
        name: &'static str,
        subject_kind: ApplicationMutationLineageSubjectKind,
        subject: &'static str,
        posture: ApplicationMutationLineagePostureKind,
        cardinality: ApplicationMutationOutputRoleCardinality,
    ) -> Self {
        Self {
            name,
            subject_kind,
            subject,
            posture,
            cardinality,
        }
    }

    pub const fn name(&self) -> &'static str {
        self.name
    }

    pub const fn subject_kind(&self) -> ApplicationMutationLineageSubjectKind {
        self.subject_kind
    }

    pub const fn subject(&self) -> &'static str {
        self.subject
    }

    pub const fn posture(&self) -> ApplicationMutationLineagePostureKind {
        self.posture
    }

    pub const fn cardinality(&self) -> ApplicationMutationOutputRoleCardinality {
        self.cardinality
    }
}

/// Installed descriptor for one entity or relation lineage output-role family.
#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub struct ApplicationMutationLineageOutputRoleFamilyDescriptor {
    prefix: &'static str,
    subject_kind: ApplicationMutationLineageSubjectKind,
    subject: &'static str,
    postures: ApplicationMutationLineagePostureSet,
    minimum: usize,
}

impl ApplicationMutationLineageOutputRoleFamilyDescriptor {
    pub(super) const fn declared(
        prefix: &'static str,
        subject_kind: ApplicationMutationLineageSubjectKind,
        subject: &'static str,
        postures: ApplicationMutationLineagePostureSet,
        minimum: usize,
    ) -> Self {
        Self {
            prefix,
            subject_kind,
            subject,
            postures,
            minimum,
        }
    }

    pub const fn prefix(&self) -> &'static str {
        self.prefix
    }

    pub const fn subject_kind(&self) -> ApplicationMutationLineageSubjectKind {
        self.subject_kind
    }

    pub const fn subject(&self) -> &'static str {
        self.subject
    }

    pub const fn postures(&self) -> ApplicationMutationLineagePostureSet {
        self.postures
    }

    pub const fn minimum(&self) -> usize {
        self.minimum
    }
}

/// Contract inventory for entity and relation lineage output roles supporting
/// `Create`, `Preserve`, `Retire`, `Split`, and `Merge`.
pub trait ApplicationMutationLineageOutputContract<Schema>: 'static
where
    Schema: ApplicationSchema,
{
    const LINEAGE_ROLES: &'static [ApplicationMutationLineageOutputRoleDescriptor];
    const LINEAGE_ROLE_FAMILIES: &'static [ApplicationMutationLineageOutputRoleFamilyDescriptor] =
        &[];
}

/// Fixed entity or relation output role carrying any of the five lineage
/// actions (`Preserve`, `Create`, `Retire`, `Split`, `Merge`).
pub trait WorthQueryApplicationLineageOutputRole: 'static {
    type Schema: ApplicationSchema;
    type Contract: ApplicationMutationLineageOutputContract<Self::Schema>;
    type Subject: WorthQueryApplicationOutputSubject<Self::Schema>;
    type Action: WorthQueryApplicationLineageOutputAction;
    type Cardinality: WorthQueryApplicationOutputCardinality;
    const NAME: &'static str;
}

/// Compile-time verified descriptor derivation for one fixed lineage output role.
pub trait WorthQueryApplicationDeclaredLineageOutputRole:
    WorthQueryApplicationLineageOutputRole
{
    const DESCRIPTOR: ApplicationMutationLineageOutputRoleDescriptor;
    const DECLARED: ();
}

impl<Role> WorthQueryApplicationDeclaredLineageOutputRole for Role
where
    Role: WorthQueryApplicationLineageOutputRole,
{
    const DESCRIPTOR: ApplicationMutationLineageOutputRoleDescriptor =
        ApplicationMutationLineageOutputRoleDescriptor::declared(
            Role::NAME,
            <Role::Subject as WorthQueryApplicationOutputSubject<Role::Schema>>::SUBJECT_KIND,
            <Role::Subject as WorthQueryApplicationOutputSubject<Role::Schema>>::SUBJECT_IDENTIFIER,
            <Role::Action as WorthQueryApplicationLineageOutputAction>::LINEAGE_POSTURE,
            <Role::Cardinality as WorthQueryApplicationOutputCardinality>::CARDINALITY,
        );

    const DECLARED: () = {
        let used = <Role as WorthQueryApplicationDeclaredLineageOutputRole>::DESCRIPTOR;
        let roles =
            <Role::Contract as ApplicationMutationLineageOutputContract<Role::Schema>>::LINEAGE_ROLES;
        let mut index = 0;
        let mut declared = false;
        while index < roles.len() {
            let role = roles[index];
            if same_text(role.name(), used.name()) {
                if !same_text(
                    role.subject_kind().canonical_name(),
                    used.subject_kind().canonical_name(),
                ) {
                    panic!("the lineage output role is declared for a different subject kind");
                }
                if !same_text(role.subject(), used.subject()) {
                    panic!("the lineage output role is declared for a different subject");
                }
                if ApplicationMutationLineagePostureSet::one(role.posture()).bits()
                    != ApplicationMutationLineagePostureSet::one(used.posture()).bits()
                {
                    panic!("the lineage output role is declared with a different posture");
                }
                if role.cardinality().admits_absence() != used.cardinality().admits_absence() {
                    panic!("the lineage output role is declared with a different cardinality");
                }
                declared = true;
            }
            index += 1;
        }
        if !declared {
            panic!("the lineage output role is not declared in its contract's LINEAGE_ROLES");
        }
    };
}

/// Variable-arity entity or relation lineage output-role family.
pub trait WorthQueryApplicationLineageOutputRoleFamily: 'static {
    type Schema: ApplicationSchema;
    type Contract: ApplicationMutationLineageOutputContract<Self::Schema>;
    type Subject: WorthQueryApplicationOutputSubject<Self::Schema>;
    const PREFIX: &'static str;
    const POSTURES: ApplicationMutationLineagePostureSet;
    const MINIMUM: usize;
}

/// Compile-time verified descriptor derivation for one lineage output-role family.
pub trait WorthQueryApplicationDeclaredLineageOutputRoleFamily:
    WorthQueryApplicationLineageOutputRoleFamily
{
    const DESCRIPTOR: ApplicationMutationLineageOutputRoleFamilyDescriptor;
    const DECLARED: ();

    fn member_name(suffix: &str) -> Result<String, WorthQueryApplicationOutputRoleNameDenial>;
}

impl<Family> WorthQueryApplicationDeclaredLineageOutputRoleFamily for Family
where
    Family: WorthQueryApplicationLineageOutputRoleFamily,
{
    const DESCRIPTOR: ApplicationMutationLineageOutputRoleFamilyDescriptor =
        ApplicationMutationLineageOutputRoleFamilyDescriptor::declared(
            Family::PREFIX,
            <Family::Subject as WorthQueryApplicationOutputSubject<Family::Schema>>::SUBJECT_KIND,
            <Family::Subject as WorthQueryApplicationOutputSubject<Family::Schema>>::SUBJECT_IDENTIFIER,
            Family::POSTURES,
            Family::MINIMUM,
        );

    const DECLARED: () = {
        let used = <Family as WorthQueryApplicationDeclaredLineageOutputRoleFamily>::DESCRIPTOR;
        let families = <Family::Contract as ApplicationMutationLineageOutputContract<
            Family::Schema,
        >>::LINEAGE_ROLE_FAMILIES;
        let mut index = 0;
        let mut declared = false;
        while index < families.len() {
            let family = families[index];
            if same_text(family.prefix(), used.prefix()) {
                if !same_text(
                    family.subject_kind().canonical_name(),
                    used.subject_kind().canonical_name(),
                ) {
                    panic!(
                        "the lineage output-role family is declared for a different subject kind"
                    );
                }
                if !same_text(family.subject(), used.subject()) {
                    panic!("the lineage output-role family is declared for a different subject");
                }
                if family.postures().bits() != used.postures().bits() {
                    panic!("the lineage output-role family is declared with different postures");
                }
                if family.minimum() != used.minimum() {
                    panic!("the lineage output-role family is declared with a different minimum");
                }
                declared = true;
            }
            index += 1;
        }
        if !declared {
            panic!("the lineage output-role family is not declared in its contract's LINEAGE_ROLE_FAMILIES");
        }
    };

    fn member_name(suffix: &str) -> Result<String, WorthQueryApplicationOutputRoleNameDenial> {
        if suffix.is_empty() {
            return Err(WorthQueryApplicationOutputRoleNameDenial::Empty);
        }
        let name = format!("{}{suffix}", Family::PREFIX);
        WorthQueryApplicationOutputRoleNameDenial::validate(&name)?;
        Ok(name)
    }
}
