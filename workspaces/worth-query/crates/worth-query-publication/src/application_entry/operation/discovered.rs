//! Ordinary discovered composition stores original meaning beside the actual native owner.
use super::*;
use crate::application_entry::mutation::authorization::WorthQueryBindingAdmission;
use crate::application_entry::mutation::{
    AdmittedDiscoveredRecoveryDenial, WorthQueryApplicationDiscoveredMutationOutcome,
    WorthQueryDiscoveredOutputRetentionFailure, WorthQueryDiscoveredProgramOutputHandle,
    WorthQueryDiscoveredProgramOutputProgress, WorthQueryDiscoveredProgramOutputSettlement,
    WorthQueryPerformedDiscoveredApplicationMutation,
    WorthQueryUnpublishedDiscoveredApplicationMutation,
};
use worth_query_declaration::facade::application_operation::ApplicationMutationAdmissionBinding;
use worth_query_declaration::facade::application_program::{
    ApplicationAction, ApplicationActionDeclaration,
    ApplicationDiscoveredActionDelivery, ApplicationDiscoveredOutputRoot,
};
use worth_query_execution::facade::domain_computation::WorthQueryProductUnpublishedRecoveryReleaseFailure;
use worth_query_execution::facade::primary_graph::{
    WorthQueryApplicationCommitReceipt, WorthQueryApplicationDiscoveredOutputConnection,
    WorthQueryManagedApplicationRecoveryPerformed,
};
mod advance;
mod batch_delivery;
mod close;
mod construction;
mod currentness;
mod custody;
mod delivery;
mod no_projection;
mod outputs;
mod recovery;
mod replay;
mod source;
use crate::application_entry::mutation::program_output_continuation::ProgramOutputContinuationFactory;
use custody::DiscoveredState as State;
pub use custody::{
    WorthQueryDiscoveredOperation, WorthQuerySettledDiscoveredOperation,
    WorthQueryTerminalDiscoveredOperation,
};

type ActionDelivery<S, I, A> = <A as ApplicationAction<S, Binding<S, I>>>::Delivery;
type Discovery<S, R> =
    <Connection<S, R> as WorthQueryApplicationDiscoveredOutputConnection<S>>::Discovery;
type DiscoveryBinding<S, R> = <Discovery<S, R> as ApplicationQueryIntent<S>>::Binding;
type DiscoveryQuery<S, R> = <DiscoveryBinding<S, R> as ApplicationQueryBinding<S>>::Query;
type DiscoveryValue<S,R> = <<DiscoveryBinding<S,R> as ApplicationQueryBinding<S>>::ResultBinding as ApplicationStructuredValueBinding>::Value;
type RootDemand<S, R> =
    <Connection<S, R> as WorthQueryApplicationDiscoveredOutputConnection<S>>::Demand;
type RootSource<S,R> = <<RootDemand<S,R> as WorthQueryApplicationOutputDemand<S>>::OutputFamily as WorthQueryProducerOutputFamily<S>>::Source;
type RootQuery<S, R> = <RootSource<S, R> as ApplicationQueryBinding<S>>::Query;
type RootValue<S,R> = <<RootSource<S,R> as ApplicationQueryBinding<S>>::ResultBinding as ApplicationStructuredValueBinding>::Value;
type NativeOutcome<S, I> = WorthQueryApplicationMutationOutcome<
    <Binding<S, I> as ApplicationMutationBinding<S>>::Denial,
    <Binding<S, I> as ApplicationMutationBinding<S>>::Result,
>;
type DiscoveredAdvance<S, I, P, R, A> = WorthQueryOperationAdvance<
    WorthQueryDiscoveredOperation<S, I, P, R, A>,
    WorthQuerySettledDiscoveredOperation<S, I, P, R, A>,
    WorthQueryTerminalDiscoveredOperation<S, I, P, R, A>,
>;

type Admission<S, I> = <Binding<S, I> as ApplicationMutationAdmissionBinding<S>>::Admission;
type DiscoveryScope<S, R> = <DiscoveryBinding<S, R> as ApplicationQueryBinding<S>>::ScopeBinding;
type DiscoveryPrincipal<S, R> =
    <DiscoveryBinding<S, R> as ApplicationQueryBinding<S>>::PrincipalIdentity;
type RootInput<S, R> = <RootSource<S, R> as ApplicationQueryBinding<S>>::Input;
type RootScope<S, R> = <RootSource<S, R> as ApplicationQueryBinding<S>>::ScopeBinding;
type RootPrincipal<S, R> = <RootSource<S, R> as ApplicationQueryBinding<S>>::PrincipalIdentity;

type ProjectedRead<S,I,A> = <ActionDelivery<S,I,A> as worth_query_declaration::facade::application_program::ApplicationDiscoveredQueryDelivery<S,Binding<S,I>>>::Read;
