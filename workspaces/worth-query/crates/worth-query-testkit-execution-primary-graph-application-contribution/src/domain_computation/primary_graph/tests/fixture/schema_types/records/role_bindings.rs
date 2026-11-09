//! Role bindings of the execution fixture.
use super::*;
pub struct PrincipalRecords;
impl fragment::Fragment for PrincipalRecords {
    const NAME: &'static str = "PrincipalRecords";
    type Bindings = PrincipalBindings;
    type Members = (Principal,);
}
pub struct PrincipalBindings;
impl fragment::BindingSet for PrincipalBindings {
    type Work<'a> = ValueDecodeAdmission<'a>;
    fn with_admission<T>(
        work: &mut Self::Work<'_>,
        run: impl FnOnce(&mut ValueDecodeAdmission<'_>) -> T,
    ) -> T {
        run(work)
    }
}
impl fragment::Scalar<u64> for PrincipalBindings {
    type Binding = U64ApplicationValueBinding;

    const WRITABLE: bool = false;
}

pub struct GrantRecords;
impl fragment::Fragment for GrantRecords {
    const NAME: &'static str = "GrantRecords";
    type Bindings = GrantBindings;
    type Members = (CapabilityGrant,);
}
pub struct GrantBindings;
impl fragment::BindingSet for GrantBindings {
    type Work<'a> = ValueDecodeAdmission<'a>;
    fn with_admission<T>(
        work: &mut Self::Work<'_>,
        run: impl FnOnce(&mut ValueDecodeAdmission<'_>) -> T,
    ) -> T {
        run(work)
    }
}
impl fragment::Scalar<u64> for GrantBindings {
    type Binding = U64ApplicationValueBinding;

    const WRITABLE: bool = true;
}
impl fragment::Scalar<String> for GrantBindings {
    type Binding = StringApplicationValueBinding;
}

pub struct ElevationRecords;
impl fragment::Fragment for ElevationRecords {
    const NAME: &'static str = "ElevationRecords";
    type Bindings = ElevationBindings;
    type Members = (CapabilityElevation,);
}
pub struct ElevationBindings;
impl fragment::BindingSet for ElevationBindings {
    type Work<'a> = ValueDecodeAdmission<'a>;
    fn with_admission<T>(
        work: &mut Self::Work<'_>,
        run: impl FnOnce(&mut ValueDecodeAdmission<'_>) -> T,
    ) -> T {
        run(work)
    }
}
impl fragment::Scalar<u64> for ElevationBindings {
    type Binding = U64ApplicationValueBinding;

    const WRITABLE: bool = true;
}
impl fragment::Scalar<String> for ElevationBindings {
    type Binding = StringApplicationValueBinding;
}

impl fragment::Scalar<GrantKey> for GrantBindings {
    type Binding = GrantKeyBinding;
}

impl fragment::Scalar<ElevationKey> for ElevationBindings {
    type Binding = ElevationKeyBinding;
}

impl fragment::Scalar<AnnotationText> for ExecutionRecordsBindings {
    type Binding = AnnotationTextBinding;
}
