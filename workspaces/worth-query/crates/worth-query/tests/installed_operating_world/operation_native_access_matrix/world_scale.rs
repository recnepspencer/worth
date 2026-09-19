use worth_query::facade::{domain, read, runtime};

use super::fixture::{
    matrix_read_declaration, matrix_semantics, NativeMatrixExecutor, NativeMatrixRead,
};
use super::samples::matrix_contract_with_override;
use crate::suite::installed_operation_fixture::ReadFamily;

macro_rules! test_domain {
    ($type:ident, $key:literal, $name:literal) => {
        #[derive(Clone, Copy, Eq, PartialEq)]
        pub(super) struct $type;

        impl domain::WorthQueryDomainEntryMarker for $type {
            fn domain_key(&self) -> &'static str {
                $key
            }

            fn display_name(&self) -> &'static str {
                $name
            }

            fn required_capability_families(
                &self,
            ) -> &'static [domain::WorthQueryCapabilityFamily] {
                &[]
            }
        }
    };
}

test_domain!(
    ForeignMatrixDomain,
    "WORTH.tests.foreign-matrix",
    "ForeignMatrix"
);
test_domain!(UnrelatedTwo, "WORTH.tests.unrelated-two", "UnrelatedTwo");
test_domain!(
    UnrelatedThree,
    "WORTH.tests.unrelated-three",
    "UnrelatedThree"
);

macro_rules! executable_matrix_domain {
    ($domain:ty) => {
        impl domain::WorthQueryExecutableDomainOperation<$domain, ReadFamily> for NativeMatrixRead {
            type Input = ();
            type Output = read::WorthQueryReadCompletion;
            type Publication = domain::WorthQueryPublishingOperation;
            type Execution = domain::WorthQueryDirectOperation;
        }

        impl domain::WorthQueryDomainOperationExecutor<$domain, NativeMatrixRead, ReadFamily>
            for NativeMatrixExecutor
        {
            const LOWERING_FAMILY: &'static str = "native-matrix-read-v1";
            const DETERMINISTIC: bool = true;
            const EXECUTION_COST: domain::WorthQueryOperationCostClass =
                domain::WorthQueryOperationCostClass::DeclaredWidth;
            const RESULT_WIDTH_COST: domain::WorthQueryOperationCostClass =
                domain::WorthQueryOperationCostClass::DeclaredWidth;

            fn installed_read_declaration(&self) -> Option<&read::WorthQueryReadDeclaration> {
                Some(matrix_read_declaration())
            }

            fn execution_resource_support(&self) -> domain::WorthQueryExecutionResourceSupport {
                super::super::installed_operation_fixture::execution_resource_support()
            }

            fn execute(
                &self,
                _: (),
                context: &domain::WorthQueryOperationExecutionContext<'_>,
                workspace: &mut domain::WorthQueryOperationWorkspace<'_>,
            ) -> Result<
                domain::WorthQueryOperationExecutionMaterial<read::WorthQueryReadCompletion>,
                domain::WorthQueryOperationExecutorFailure,
            > {
                Ok(domain::WorthQueryOperationExecutionMaterial::new(
                    context.execute_installed_read(workspace)?,
                    domain::WorthQueryOperationResultState::Ready,
                ))
            }
        }
    };
}

executable_matrix_domain!(ForeignMatrixDomain);
executable_matrix_domain!(UnrelatedTwo);
executable_matrix_domain!(UnrelatedThree);

pub(super) fn add_unrelated_domains(
    builder: worth_query::facade::consumer_kit::WorthQueryInMemoryTestRuntimeBuilder,
) -> worth_query::facade::consumer_kit::WorthQueryInMemoryTestRuntimeBuilder {
    let same_contract = domain::WorthQueryDomainOperationDefinition::<
        ForeignMatrixDomain,
        NativeMatrixRead,
        ReadFamily,
    >::new(
        domain::WorthQueryDomainOperationIdentity::new("foreign-native-matrix-read", 1),
        matrix_semantics(matrix_contract_with_override(1, None)),
    );
    let revision_drift = domain::WorthQueryDomainOperationDefinition::<
        UnrelatedTwo,
        NativeMatrixRead,
        ReadFamily,
    >::new(
        domain::WorthQueryDomainOperationIdentity::new("revision-native-matrix-read", 1),
        matrix_semantics(matrix_contract_with_override(2, None)),
    );
    let family_drift = domain::WorthQueryDomainOperationDefinition::<
        UnrelatedThree,
        NativeMatrixRead,
        ReadFamily,
    >::new(
        domain::WorthQueryDomainOperationIdentity::new("family-native-matrix-read", 1),
        matrix_semantics(matrix_contract_with_override(
            1,
            Some((2, read::ScalarAspectType::String)),
        )),
    );
    builder
        .domain_package(
            domain::WorthQueryDomainPackage::declare(
                ForeignMatrixDomain,
                domain_identity("foreign-matrix"),
            )
            .operation(same_contract),
        )
        .domain_package(
            domain::WorthQueryDomainPackage::declare(
                UnrelatedTwo,
                domain_identity("unrelated-two"),
            )
            .operation(revision_drift),
        )
        .domain_package(
            domain::WorthQueryDomainPackage::declare(
                UnrelatedThree,
                domain_identity("unrelated-three"),
            )
            .operation(family_drift),
        )
        .domain_operation_executor(
            ForeignMatrixDomain,
            NativeMatrixRead,
            ReadFamily,
            NativeMatrixExecutor,
        )
        .domain_operation_executor(
            UnrelatedTwo,
            NativeMatrixRead,
            ReadFamily,
            NativeMatrixExecutor,
        )
        .domain_operation_executor(
            UnrelatedThree,
            NativeMatrixRead,
            ReadFamily,
            NativeMatrixExecutor,
        )
}

pub(super) fn bind_foreign_same_contract(
    workspace: &runtime::WorthQueryWorkspace,
) -> domain::WorthQueryBoundDomainOperation<
    ForeignMatrixDomain,
    NativeMatrixRead,
    ReadFamily,
    worth_query::facade::foundation::ObservationLaneWitness,
> {
    let installed = workspace.domain(ForeignMatrixDomain).unwrap();
    workspace
        .observe_operating_world(workspace.current_world())
        .unwrap()
        .family(ReadFamily)
        .bind(&installed, NativeMatrixRead)
        .unwrap()
}

pub(super) fn bind_foreign_revision(
    workspace: &runtime::WorthQueryWorkspace,
) -> domain::WorthQueryBoundDomainOperation<
    UnrelatedTwo,
    NativeMatrixRead,
    ReadFamily,
    worth_query::facade::foundation::ObservationLaneWitness,
> {
    let installed = workspace.domain(UnrelatedTwo).unwrap();
    workspace
        .observe_operating_world(workspace.current_world())
        .unwrap()
        .family(ReadFamily)
        .bind(&installed, NativeMatrixRead)
        .unwrap()
}

pub(super) fn bind_foreign_family(
    workspace: &runtime::WorthQueryWorkspace,
) -> domain::WorthQueryBoundDomainOperation<
    UnrelatedThree,
    NativeMatrixRead,
    ReadFamily,
    worth_query::facade::foundation::ObservationLaneWitness,
> {
    let installed = workspace.domain(UnrelatedThree).unwrap();
    workspace
        .observe_operating_world(workspace.current_world())
        .unwrap()
        .family(ReadFamily)
        .bind(&installed, NativeMatrixRead)
        .unwrap()
}

pub(super) fn assert_unrelated_domains_installed(workspace: &runtime::WorthQueryWorkspace) {
    assert!(workspace.domain(ForeignMatrixDomain).is_ok());
    assert!(workspace.domain(UnrelatedTwo).is_ok());
    assert!(workspace.domain(UnrelatedThree).is_ok());
}

fn domain_identity<D>(name: &str) -> domain::WorthQueryDomainIdentityDeclaration<D> {
    domain::WorthQueryDomainIdentityDeclaration::new(
        domain::WorthQueryDomainIdentityNamespace::new("WORTH.tests").unwrap(),
        domain::WorthQueryDomainIdentityName::new(name).unwrap(),
        domain::WorthQueryDomainSemanticVersion::new(1, 0),
    )
}
