//! Distinct runtime mutation binding over the original operation and output role.
use crate::application_invariant_acceptance::managed_index::maintained as catalog;
use catalog::generated::operation::{
    Decision, Denial, DenialBinding, Generate, GenerateItem, Generated, InputBinding, Outputs,
    ResultBinding,
};
use catalog::schema::*;
use worth_query_decl::facade::{
    application_operation::*, application_query::*, application_schema::*,
};
use worth_query_host::facade::primary_graph::*;
pub struct Binding;
impl ApplicationMutationAdmissionBinding<Catalog> for Binding {
    type Admission = ApplicationConventionalAdmission;
}
impl ApplicationMutationBinding<Catalog> for Binding {
    type Input = Generate;
    type InputBinding = InputBinding;
    type Result = Generated;
    type ResultBinding = ResultBinding;
    type IdempotencyKey = u64;
    type Operation = GenerateItem;
    type Decision = Decision;
    type Denial = Denial;
    type DenialBinding = DenialBinding;
    type Output = Outputs;
    type ScopeBinding = ApplicationMutationFieldScope<DirectoryIdentity>;
    type PrincipalBinding = CatalogPrincipal;
    type Mapping = ExternalMapping;
    type Principal = Principal;
    type PrincipalIdentity = u64;
    type PrincipalIdentityBinding = U64ApplicationValueBinding;
    type SourceExpectation = ApplicationQueryMutationSource<catalog::geometry::query::BoxQuery>;
    const IDENTITY: &'static str =
        "maintained-generated.applied-nested-source.operation-binding.v1";
    const HANDLER_IDENTITY: &'static str = "maintained-generated.applied-nested-source.handler.v1";
    const IDEMPOTENCY_IDENTITY: &'static str =
        "maintained-generated.applied-nested-source.idempotency.v1";
    const REQUIRES_APPLICATION_PROGRAM: bool = true;
    fn workflow_execution_requirements() -> ApplicationWorkflowExecutionRequirements {
        ApplicationWorkflowExecutionRequirements::not_applicable()
    }
    fn scope_field() -> DirectoryIdentity {
        DirectoryIdentity::at("DirectoryIdentity")
    }
    fn principal_binding() -> ApplicationPrincipalBindingRef<
        Catalog,
        CatalogPrincipal,
        ExternalMapping,
        Principal,
        u64,
        U64ApplicationValueBinding,
    > {
        CatalogPrincipal::reference()
    }
    fn expected_source_parameters(
        input: &Generate,
    ) -> Result<
        Option<ApplicationQueryParameterSet<catalog::geometry::query::BoxQuery>>,
        ApplicationValueEncodeDenial,
    > {
        Ok(Some(
            <catalog::geometry::query::FindBoxes as ApplicationQueryIntent<Catalog>>::parameters(
                &catalog::geometry::query::FindBoxes {
                    directory: input.directory.clone(),
                    probe: catalog::bounds::Coordinates::new(-1, -1, 1, 1),
                },
            ),
        ))
    }
}

impl ApplicationMutationInputScope<Catalog, Binding> for Generate {
    fn scope_for_binding(&self) -> <Binding as ApplicationMutationBinding<Catalog>>::ScopeBinding {
        ApplicationMutationFieldScope::new(
            DirectoryIdentity::at("DirectoryIdentity"),
            self.directory.clone(),
        )
    }
}

thread_local! {
    static OBSERVED: std::cell::RefCell<Option<Vec<WorthQueryCurrentOutputSelection<Catalog, Item>>>> = const { std::cell::RefCell::new(None) };
}
pub(super) struct Observation;
impl Observation {
    pub(super) fn open() -> Self {
        OBSERVED.with(|records| {
            assert!(records.borrow().is_none());
            *records.borrow_mut() = Some(Vec::new());
        });
        Self
    }
    pub(super) fn take(&self) -> Vec<WorthQueryCurrentOutputSelection<Catalog, Item>> {
        OBSERVED.with(|records| std::mem::take(records.borrow_mut().as_mut().unwrap()))
    }
}
impl Drop for Observation {
    fn drop(&mut self) {
        OBSERVED.with(|records| *records.borrow_mut() = None);
    }
}

pub(super) struct Handler;
impl OperationHandler<Catalog, Binding> for Handler {
    fn decide(
        &self,
        input: &Generate,
        reader: &mut DecisionReader<'_, '_, '_, Catalog, Binding>,
    ) -> HandlerResult<Decision, Denial> {
        let selection = (|| {
            let directory = reader.resolve_entity(
                DirectoryIdentity::at("DirectoryIdentity"),
                input.directory.clone(),
            )?;
            reader.current_output::<super::Family, Directory>(&directory)
        })();
        match selection {
            Ok(selection) => OBSERVED.with(|records| {
                if let Some(records) = records.borrow_mut().as_mut() {
                    records.push(selection);
                }
            }),
            Err(cause) => return HandlerResult::ExecutionDenied(cause),
        }
        <catalog::generated::operation::Handler as OperationHandler<Catalog, Binding>>::decide(
            &catalog::generated::operation::Handler,
            input,
            reader,
        )
    }
    fn build_candidate(
        &self,
        input: &Generate,
        decision: Decision,
        writer: &mut CandidateWriter<'_, Catalog, Binding>,
    ) -> HandlerResult<Generated, Denial> {
        <catalog::generated::operation::Handler as OperationHandler<Catalog, Binding>>::build_candidate(
            &catalog::generated::operation::Handler, input, decision, writer,
        )
    }
}
