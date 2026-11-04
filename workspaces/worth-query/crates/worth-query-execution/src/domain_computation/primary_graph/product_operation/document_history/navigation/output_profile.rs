//! Installed rebuild correspondence; an index addresses immutable owner metadata.
use super::super::{HistoryModelIdentity, WorthQueryDocumentHistory};
use super::*;
use worth_query_declaration::facade::{
    application_history::{
        ApplicationHistoryAccessBinding, ApplicationHistoryBinding, ApplicationHistoryModel,
    },
    application_program::{
        ApplicationConnectionShape, ApplicationOutputGraphShape, ApplicationProgramDefinition,
        ApplicationProgramRevision,
    },
    application_query::{ApplicationQueryBinding, ApplicationQueryScopeBinding},
};
use worth_query_installation::facade::ApplicationSchema;
use {
    crate::domain_computation::primary_graph::application_installation::WorthQueryProgramApplicationRuntime,
    crate::domain_computation::primary_graph::application_installation::WorthQueryProgramOwner,
    crate::domain_computation::primary_graph::request_execution::WorthQueryAdvancementPhase,
    crate::domain_computation::primary_graph::WorthQueryApplicationOutputDemand,
    crate::domain_computation::primary_graph::WorthQueryApplicationRequiredOutputConnection,
    crate::domain_computation::primary_graph::WorthQueryProducerOutputFamily,
};

type Connection<S,R> = <<R as ApplicationOutputGraphShape<S>>::RootConnection as ApplicationConnectionShape<S>>::Binding;
type Demand<S, R> = <Connection<S, R> as WorthQueryApplicationRequiredOutputConnection<S>>::Demand;
type Source<S,R> = <<Demand<S,R> as WorthQueryApplicationOutputDemand<S>>::OutputFamily as WorthQueryProducerOutputFamily<S>>::Source;

/// Rebuild meaning bound to one actual document and immutable installed record.
/// It carries no current READ or NAV permission and cannot create output work.
#[derive(Clone)]
pub struct WorthQueryHistoryOutputProfile {
    pub(super) model: HistoryModelIdentity,
    pub(super) program: ApplicationProgramRevision,
    pub(super) program_type: std::any::TypeId,
    pub(super) root: std::any::TypeId,
    pub(super) recipe: std::any::TypeId,
    pub(super) delivery_index: usize,
    source_intent: crate::domain_computation::primary_graph::WorthQueryHistoryOutputIntent,
}

impl WorthQueryHistoryOutputProfile {
    fn same_header(&self, other: &Self) -> bool {
        self.model == other.model
            && self.program == other.program
            && self.program_type == other.program_type
            && self.root == other.root
            && self.recipe == other.recipe
            && self.delivery_index == other.delivery_index
    }

    /// Guarded consumers require the original immutable registered custody.
    /// Independent equal descriptions first pass funded owner readmission.
    pub(in crate::domain_computation::primary_graph) fn same_custody(&self, other: &Self) -> bool {
        self.same_header(other) && self.source_intent.same_custody(&other.source_intent)
    }

    pub(super) fn equivalent(
        &self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        other: &Self,
    ) -> Result<bool, worth_relational::facade::transactions::RelationalExecutionDenialCause> {
        if !self.same_header(other) {
            return Ok(false);
        }
        self.source_intent
            .equivalent(execution, &other.source_intent)
    }
    pub(in crate::domain_computation::primary_graph) fn source_intent(
        &self,
    ) -> &crate::domain_computation::primary_graph::WorthQueryHistoryOutputIntent {
        &self.source_intent
    }
    pub(in crate::domain_computation::primary_graph) fn matches_program<
        S: ApplicationSchema,
        P: ApplicationProgramDefinition<S>,
    >(
        &self,
        program: &WorthQueryProgramApplicationRuntime<S, P>,
    ) -> bool {
        self.program_type == crate::equality_type_id::EqualityTypeId::of::<P>().into_external()
            && &self.program == program.owned_revision()
    }
}

/// Original D is disclosed to the output driver only after current READ,
/// metadata and NAV have admitted this Native advancement.
pub enum WorthQueryHistoryOutputNavigationAdvance<'authority, 'scope> {
    Pending,
    Outputs {
        branch: crate::basis::WorthQueryProductBranch,
        source: Arc<crate::domain_computation::primary_graph::WorthQueryApplicationReadObservation>,
        authority: &'authority WorthQueryHistoryOutputAuthority<'authority, 'scope>,
    },
    Selecting {
        branch: crate::basis::WorthQueryProductBranch,
        source: Arc<crate::domain_computation::primary_graph::WorthQueryApplicationReadObservation>,
        authority: &'authority WorthQueryHistoryOutputAuthority<'authority, 'scope>,
    },
    Selected(WorthQueryHistoryNavigationCompletion),
}

impl<S: ApplicationSchema + 'static, P: ApplicationProgramDefinition<S>>
    WorthQueryProgramApplicationRuntime<S, P>
{
    /// Resolves the complete installed delivery corresponding to this history.
    /// Publication fixes Recipe and retains its original constructed demand.
    pub fn bind_history_output_profile<H, R, Recipe>(
        &self,
        phase: &WorthQueryAdvancementPhase<'_>,
        _: &crate::publication_boundary::WorthQueryProgramPublicationAccess,
        document: &WorthQueryDocumentHistory<'_, S, H>,
        source_intent: crate::domain_computation::primary_graph::WorthQueryHistoryOutputIntent,
    ) -> Result<WorthQueryHistoryOutputProfile, WorthQueryHistoryNavigationDenial>
    where
        H: ApplicationHistoryBinding<S>,
        R: ApplicationOutputGraphShape<S>,
        Connection<S, R>: WorthQueryApplicationRequiredOutputConnection<S>,
        Recipe: 'static,
    {
        use WorthQueryHistoryNavigationDenial as Denial;
        use WorthQueryHistoryNavigationEligibilityDenial as Eligibility;
        let application = document.application();
        if !std::ptr::eq(self.runtime(), application) {
            return Err(Denial::Document(
                super::super::WorthQueryDocumentHistoryDenial::ForeignModel,
            ));
        }
        phase
            .check_live_for(&application.runtime.product_runtime)
            .map_err(Denial::Stopped)?;
        if crate::equality_type_id::EqualityTypeId::of::<<<Source<S,R> as ApplicationQueryBinding<S>>::ScopeBinding as ApplicationQueryScopeBinding<S>>::Scope>().into_external()
            != crate::equality_type_id::EqualityTypeId::of::<ApplicationHistoryModel<S,H>>().into_external()
        {
            return Err(Denial::Document(super::super::WorthQueryDocumentHistoryDenial::ForeignModel));
        }
        let source_binding = application
            .installed_schema()
            .installed_query_binding::<Source<S, R>>()
            .map_err(|_| Denial::RequiresSemanticCorrection(Eligibility::RequiredOutputs))?;
        if !source_intent.matches_binding(&document.model, &source_binding) {
            return Err(Denial::IntentDrift);
        }
        let selected = application
            .on_branch(document.branch())
            .select(phase)
            .map_err(Denial::ProductAdmission)?;
        let owner = self
            .selected_program_owner(&selected)
            .map_err(|_| Denial::RequiresSemanticCorrection(Eligibility::ProgramUnavailable))?;
        if owner.owned_revision() != self.owned_revision() {
            return Err(Denial::RequiresSemanticCorrection(
                Eligibility::ProgramChanged,
            ));
        }
        let installed = application
            .installed_program_support()
            .and_then(|support| support.native_program(owner.owned_revision()))
            .ok_or(Denial::RequiresSemanticCorrection(
                Eligibility::ProgramUnavailable,
            ))?;
        let access = ApplicationHistoryAccessBinding::<S, H>::descriptor();
        let history = application
            .installed_schema()
            .installed_history::<H>()
            .map_err(|_| Denial::RequiresSemanticCorrection(Eligibility::RequiredOutputs))?;
        let connection = R::RootConnection::declaration();
        let connections = R::connections();
        let delivery_index = installed
            .installed
            .action_inventory()
            .declarations()
            .iter()
            .position(|action| {
                action.mutation_binding_type().is_some_and(|binding| {
                    history
                        .operation_descriptors()
                        .iter()
                        .any(|operation| operation.binding_type() == binding)
                }) && action
                    .delivery()
                    .and_then(|delivery| delivery.output())
                    .is_some_and(|output| {
                        !output.is_discovered()
                            && output.root_type()
                                == crate::equality_type_id::EqualityTypeId::of::<R>()
                                    .into_external()
                            && output.read().is_none_or(|read| read == &access)
                            && output.root_connection() == &connection
                            && output.root_connections() == connections
                    })
            })
            .ok_or(Denial::RequiresSemanticCorrection(
                Eligibility::RequiredOutputs,
            ))?;
        Ok(WorthQueryHistoryOutputProfile {
            model: document.model.clone(),
            program: *owner.owned_revision(),
            program_type: crate::equality_type_id::EqualityTypeId::of::<P>().into_external(),
            root: crate::equality_type_id::EqualityTypeId::of::<R>().into_external(),
            recipe: crate::equality_type_id::EqualityTypeId::of::<Recipe>().into_external(),
            delivery_index,
            source_intent,
        })
    }
}
