use std::marker::PhantomData;

pub trait WritePosture {
    const WRITABLE: bool;
}

pub trait WritableCapability: WritePosture {}

pub trait OperationRequiresAbility<Operation> {}

/// Compile-time declaration that an application member may inform an
/// operation's decision without widening its installed read contract.
pub trait OperationReads<Operation> {}

/// Compile-time declaration that a field may carry an expected-version
/// precondition for an operation. Installed authority remains mandatory.
pub trait OperationExpectsVersion<Operation> {}

/// Compile-time declaration that a field may carry an expected-fact
/// precondition for an operation. Installed authority remains mandatory.
pub trait OperationExpectsFact<Operation> {}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ReadOnly;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ReadWrite;

impl WritableCapability for ReadWrite {}

impl WritePosture for ReadOnly {
    const WRITABLE: bool = false;
}

impl WritePosture for ReadWrite {
    const WRITABLE: bool = true;
}

pub trait EqualityPosture {
    const QUERYABLE: bool;
}

pub trait EqualityCapable: EqualityPosture {}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct NoEqualityPredicate;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct EqualityPredicate;

impl EqualityCapable for EqualityPredicate {}

impl EqualityPosture for NoEqualityPredicate {
    const QUERYABLE: bool = false;
}

impl EqualityPosture for EqualityPredicate {
    const QUERYABLE: bool = true;
}

/// Compile-time declaration that a field marker may be written by an operation
/// marker. This is descriptive only; installed operation authority is still
/// required later.
pub trait OperationWrites<Operation> {}

/// Aspect removal is distinct from every child field write; installation still admits it.
pub trait OperationRemovesAspects<Operation> {}

/// Compile-time declaration that an entity marker may be created by an
/// operation marker.
pub trait OperationCreates<Operation> {}

pub trait CreatableBy<Operation>: OperationCreates<Operation> {}

impl<Entity, Operation> CreatableBy<Operation> for Entity where Entity: OperationCreates<Operation> {}

pub trait OperationDeletes<Operation> {}

pub trait OperationLinks<Operation> {}

pub trait OperationUnlinks<Operation> {}

/// Compile-time declaration that an effect marker may be emitted by an
/// operation marker.
pub trait OperationEmits<Operation> {}

pub trait ApplicationUnitMarker<DomainUnit> {
    const NAME: &'static str;
}

pub trait ApplicationFieldUnit {
    const NAME: Option<&'static str>;
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct NoApplicationUnit;

impl ApplicationFieldUnit for NoApplicationUnit {
    const NAME: Option<&'static str> = None;
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct DeclaredApplicationUnit<Marker, DomainUnit>(PhantomData<fn() -> (Marker, DomainUnit)>);

impl<Marker, DomainUnit> ApplicationFieldUnit for DeclaredApplicationUnit<Marker, DomainUnit>
where
    Marker: ApplicationUnitMarker<DomainUnit>,
{
    const NAME: Option<&'static str> = Some(Marker::NAME);
}
