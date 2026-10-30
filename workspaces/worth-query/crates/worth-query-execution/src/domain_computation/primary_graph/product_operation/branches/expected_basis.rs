//! Description readmission composes the existing World and Native receivers.
use super::*;
use crate::basis::{
    WorthQueryExpectedProductBasisDescription, WorthQueryProductBranchAdmissionDenial,
};
use crate::domain_computation::primary_graph::{
    WorthQueryAdvancementDenial, WorthQueryAdvancementPhase, WorthQuerySelectedProductOperation,
};
use worth_relational::facade::branch::RelationalBranchBasisDenial;

#[derive(Debug)]
pub enum WorthQueryExpectedProductBasisSelectionDenial {
    UnsupportedDescriptionVersion {
        supported: u16,
        actual: u16,
    },
    Admission(WorthQueryProductBranchAdmissionDenial),
    NativeReadmission(RelationalBranchBasisDenial),
    NativeCurrentness(RelationalBranchBasisDenial),
    /// The admitted Native basis is not the tuple selected by World's commit.
    ComponentMismatch,
    Stopped(WorthQueryAdvancementDenial),
}

impl<'runtime, Schema: ApplicationSchema> WorthQueryApplicationProductBranches<'runtime, Schema> {
    /// Ask both owners to readmit stored comparison data. No newer selection
    /// replaces a moved source. Ordinary requests still govern domain disclosure.
    pub fn select_expected(
        &self,
        description: &WorthQueryExpectedProductBasisDescription,
        phase: &WorthQueryAdvancementPhase<'_>,
    ) -> Result<
        WorthQuerySelectedProductOperation<'runtime, Schema>,
        WorthQueryExpectedProductBasisSelectionDenial,
    > {
        use WorthQueryExpectedProductBasisSelectionDenial as Denial;
        let application = self.application;
        let execution = phase
            .execution_request_for(&application.product_runtime)
            .map_err(|_| {
                Denial::Admission(WorthQueryProductBranchAdmissionDenial::Refused(
                    crate::basis::WorthQueryProductBranchAdmissionRefusalKind::ForeignPhase,
                ))
            })?;
        check_live(&execution)?;
        let (supported, actual) = description.version();
        if actual != supported {
            return Err(Denial::UnsupportedDescriptionVersion { supported, actual });
        }
        let product = application
            .admit_with_observation_room(|| {
                application
                    .product_runtime
                    .readmit_product_reference(description.world())
            })
            .map_err(Denial::Admission)?;
        // Native readmission also admits retained history. Currentness must
        // therefore be checked by that same owner before composing the selection.
        let (native, currentness) = application
            .primary_provider
            .graph
            .with_runtime(&execution, |runtime| {
                let port = runtime.owner_component_services().basis_port();
                port.readmit_branch_basis(description.native())
                    .map(|native| {
                        let currentness = port.compare_current_exact(&native);
                        (native, currentness)
                    })
            })
            .map_err(Denial::NativeReadmission)?;
        // Retention destructors stay outside borrowed Native access on every path.
        currentness.map_err(Denial::NativeCurrentness)?;
        if native.descriptor() != product.relational_basis().descriptor() {
            return Err(Denial::ComponentMismatch);
        }
        drop(native);
        check_live(&execution)?;
        application
            .on_product(&execution, product)
            .map_err(Denial::Admission)
    }
}

impl<Schema> WorthQuerySelectedProductOperation<'_, Schema> {
    pub fn describe_basis(&self) -> WorthQueryExpectedProductBasisDescription {
        WorthQueryExpectedProductBasisDescription::from_observation(self.product().observation())
    }
}

fn check_live(
    execution: &worth_execution::ExecutionRequest<'_, '_>,
) -> Result<(), WorthQueryExpectedProductBasisSelectionDenial> {
    execution.check_live().map_err(|stop| {
        WorthQueryExpectedProductBasisSelectionDenial::Stopped(
            crate::domain_computation::primary_graph::application_contribution::advancement_denial(
                worth_execution::WorkCeilingDenial::Stopped(stop),
            ),
        )
    })
}
