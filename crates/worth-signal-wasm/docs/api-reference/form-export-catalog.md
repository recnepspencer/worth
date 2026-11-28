# Complete Form Export Catalog

> **Ask your AI agent:**
>
> “I’m building: [describe your form and requirements]. Before proposing an
> implementation, explore the complete Worth Signals Forms surface. Find the
> most effective construction the framework supports. Identify adjacent
> capabilities that would materially improve correctness, user experience,
> accessibility, resilience, inspection, or maintainability based on the
> requirements.  Inspect the source code where necessary if anything from the
> form export catalog seems useful.
>
> Separate your recommendations into:
> essential foundations;
> high-leverage improvements;
> optional broader architectural integrations, that may tie into resource
> lines, host setup, routes, etc.
>
> Explain the important authority and lifecycle boundaries, current
> limitations, and tradeoffs. Propose 2-3 design patterns, their requirements,
> scope, and tradeoffs.”

This page gives every public Forms declaration a short, plain-English job
description. It is an orientation map, not a substitute for the task-focused
guides or the exact TypeScript signatures.

At this time, the deeper details are discoverable in source code. Start with
the declaration file named above each group in `package/types/forms/`, then
follow its imports or implementation into `package-src/product/forms/`. The
ordinary application entry point remains `signals.form(...)`; many entries
below are report, evidence, policy, or host-integration types rather than
things an application constructs directly.

## Accessibility

Source declaration: `package/types/forms/accessibility.d.ts`

- `FormAccessibilityAnnouncementPriority` — Selects whether assistive announcements are off, polite, or assertive.
- `FormAccessibilityFieldArtifact` — Describes one field's labels, relationships, state, ordering, and focus support.
- `FormAccessibilityMessageArtifact` — Describes how one form message is exposed to assistive technology.
- `FormAccessibilityReport` — Collects accessibility artifacts, focus guidance, ordering hints, and scan costs.
- `FormAccessibilitySectionArtifact` — Describes an accessible field group and its position in the form.

## Action Authoring

Source declaration: `package/types/forms/action_authoring.d.ts`

- `FormActionDeclaration` — Defines a named form action and the policy governing its execution.
- `FormActionDeclarationOptions` — Configures an ordinary application-owned form action.
- `FormActionEffectPolicy` — Selects how an action exposes and settles its external effect.
- `FormActionIdempotency` — States whether repeated action execution is safe or identity-bound.
- `FormActionKind` — Identifies the declared category of a form action.
- `FormActionPatchPolicy` — Controls whether and how an action uses the current patch plan.
- `FormActionResultKind` — Names the result states an authored action can produce.
- `FormActionsBuilder` — Maps action names to their authored declarations.
- `FormActionsFactory` — Supplies helpers for declaring submit, custom, resource, and step actions.
- `FormResourceActionDeclaration` — Unites the supported resource-backed action declaration variants.
- `FormResourceBackedLifecycleActionDeclarationOptions` — Configures a resource refresh or revalidation action.
- `FormResourceBackedPatchActionDeclarationOptions` — Configures a resource action that lowers the form patch plan.
- `FormResourceBackedRecoveryActionDeclarationOptions` — Configures an exact replay, restore, or rollback action.
- `FormResourcePatchActionDeclaration` — Declares a resource-backed action that applies form changes.
- `FormResourceRefreshActionDeclaration` — Declares a resource refresh action.
- `FormResourceReplayExactActionDeclaration` — Declares exact replay against retained resource history.
- `FormResourceRestoreExactActionDeclaration` — Declares exact restoration of a retained resource snapshot.
- `FormResourceRevalidateActionDeclaration` — Declares resource revalidation against its current authority.
- `FormResourceRollbackLastEffectActionDeclaration` — Declares rollback of the resource line's latest effect.
- `FormStepActionCommand` — Names the step movement an action requests.
- `FormStepActionDeclarationOptions` — Configures an action that moves through a multi-step form.

## Action Inspection And Execution

Source declaration: `package/types/forms/actions.d.ts`

- `FormActionCatalogEntry` — Summarizes one declared action for inspection and tooling.
- `FormActionDebugReport` — Explains an action's declaration, readiness, plan, and recent outcomes.
- `FormActionExecutionArtifact` — Records an admitted action execution and its operation identity.
- `FormActionExecutionResultKind` — Names the immediate outcome of executing an action.
- `FormActionPlan` — Describes what an action would do without performing the effect.
- `FormActionRecovery` — Describes the recovery option attached to an action result.
- `FormActionRecoveryKind` — Names the recovery strategy available after an action outcome.
- `FormActionResultArtifact` — Records the settled result of an action operation.
- `FormActionsReport` — Aggregates declared actions, readiness, execution state, and scan costs.
- `FormServerMessageArtifact` — Carries a server-authored message admitted through action settlement.

## Admission

Source declaration: `package/types/forms/admission.d.ts`

- `FormAdmissionArtifact` — Records the outcome and evidence of one admission decision.
- `FormAdmissionBindingEvidence` — Proves which capability binding supported an admission result.
- `FormAdmissionBuilder` — Maps admission names to their declarations.
- `FormAdmissionCapability` — Names a capability that may be required before form work proceeds.
- `FormAdmissionContext` — Exposes safe form facts to an admission rule.
- `FormAdmissionFactory` — Supplies helpers for declaring form admission requirements.
- `FormAdmissionPosture` — Names whether an admission requirement is allowed, blocked, or unavailable.
- `FormAdmissionReport` — Aggregates current admission decisions, blockers, evidence, and costs.
- `FormCurrentAdmissionBinding` — Describes the capability binding currently satisfying a requirement.

## Attachment Transfers

Source declaration: `package/types/forms/attachment_transfers.d.ts`

- `FormAttachmentTransferFieldReport` — Reports upload, processing, and download state for one attachment field.
- `FormAttachmentTransfersReport` — Aggregates attachment transfer state and costs across the form.

## Attachments

Source declaration: `package/types/forms/attachments.d.ts`

- `FormAttachmentPresentationArtifact` — Admits attachment presentation state from the host boundary.
- `FormAttachmentsReport` — Aggregates attachment presentation artifacts and their current status.

## Availability

Source declaration: `package/types/forms/availability.d.ts`

- `FormAvailabilityArtifact` — Records the evaluated availability of one form target.
- `FormAvailabilityBuilder` — Maps availability names to their declarations.
- `FormAvailabilityContext` — Exposes form, field, host, and admission facts to availability rules.
- `FormAvailabilityFactory` — Supplies helpers for declaring availability rules.
- `FormAvailabilityReport` — Aggregates enabled, disabled, hidden, and read-only decisions with evidence.
- `FormAvailabilityScope` — Names the form target governed by an availability rule.
- `FormAvailabilityState` — Names the resulting enabled, disabled, hidden, or read-only state.
- `FormDraftAvailabilityPolicy` — Controls how draft state participates in availability evaluation.

## Canonicalization

Source declaration: `package/types/forms/canonicalization.d.ts`

- `FormCanonicalizationArtifact` — Records a host-reported canonical value or canonicalization failure.

## Collaboration

Source declaration: `package/types/forms/collaboration.d.ts`

- `FormCollaborationArtifact` — Records one admitted collaboration update for the form.
- `FormCollaborationComment` — Describes a collaboration comment attached to a form target.
- `FormCollaborationDeclaration` — Configures the form's collaboration mode and identity policy.
- `FormCollaborationEvent` — Describes a presence, lease, comment, or change event from a collaborator.
- `FormCollaborationEventKind` — Names the supported categories of collaboration event.
- `FormCollaborationLease` — Describes temporary collaborator ownership of a form target.
- `FormCollaborationMode` — Selects whether collaboration is disabled, advisory, or enforced.
- `FormCollaborationPresence` — Describes a collaborator currently present in the form.
- `FormCollaborationReport` — Aggregates collaboration state, proof, history, and scan costs.
- `FormCollaborationResourceProof` — Connects a collaboration update to compatible resource authority.

## Controller And Declaration

Source declaration: `package/types/forms/controller.d.ts`

- `FormController` — Exposes the complete runtime API for one declared form instance.
- `FormDeclaration` — Defines a form's source, fields, policies, actions, and integrations.
- `FormFactory` — Constructs typed forms and exposes explicit source factories.
- `FormFieldHandleFor` — Resolves a field declaration to its correctly typed runtime handle.

## Core State And Fields

Source declaration: `package/types/forms/core.d.ts`

- `CallableFormSignal` — Represents a readable form value that can also be called like a signal.
- `FormAttachmentFieldHandle` — Controls draft state and identity for an attachment field.
- `FormAttachmentIdentity` — Describes the stable identity extracted from an attachment value.
- `FormBaseFieldHandle` — Defines the shared reads, writes, input, interaction, and diagnostics API for fields.
- `FormDirtyState` — Summarizes whether the form differs from its current source value.
- `FormEvidenceFieldHandle` — Controls an evidence field whose value carries verifiable attachment identity.
- `FormFieldDiagnostics` — Explains one field's source, draft, input, equality, and validation state.
- `FormFieldDirtyState` — Describes whether one field changed and how that conclusion was reached.
- `FormFieldHandle` — Unites scalar, repeated, attachment, and evidence field handles.
- `FormFieldLocus` — Identifies a field's semantic location within the form.
- `FormFieldWritePosture` — Reports whether a field write is admitted and why.
- `FormInputAdapterDiagnostics` — Explains an input adapter's parsing, composition, and capability posture.
- `FormPatchOperation` — Describes one field-level operation in a proposed form patch.
- `FormPatchPlan` — Collects the descriptive changes between source and effective form state.
- `FormPatchReplacement` — Describes a whole-value replacement when narrow patching is unavailable.
- `FormReadinessBlocker` — Explains one reason the form or an action cannot proceed.
- `FormRepeatedCollectionIdentity` — Reports stable item identities for a repeated field.
- `FormRepeatedFieldHandle` — Adds identity-aware collection editing to the common field API.
- `FormScalarFieldHandle` — Controls draft state and input for an ordinary scalar field.
- `FormSemanticEqualityCounters` — Exposes the work performed by semantic equality checks.
- `FormSource` — Names the supported live, graph, resource, readable, and snapshot source shapes.
- `FormSourceBootstrapArtifact` — Records whether source startup or restoration is ready.
- `FormSourceBootstrapStatus` — Names the ready, pending, or unavailable source startup state.
- `FormSourceDescriptor` — Describes the declared kind and identity of a form source.
- `FormSourceMigrationResult` — Records the outcome of moving draft state to a changed source.
- `FormSourceSchemaContext` — Supplies source metadata used to validate and migrate form state.
- `FormSourceValue` — Extracts the value type represented by a form source.

## Exit

Source declaration: `package/types/forms/exit.d.ts`

- `FormExitPresentationArtifact` — Admits host presentation state for leaving the form.
- `FormExitReport` — Aggregates current exit presentation state and history.

## Field Authoring

Source declaration: `package/types/forms/field_authoring.d.ts`

- `FormAttachmentIdentityOptions` — Configures how an attachment or evidence value yields stable identity.
- `FormFieldAccessibilityOptions` — Declares labels, descriptions, ordering, and announcement behavior for a field.
- `FormFieldDeclaration` — Defines one field's family, path, defaults, and policies.
- `FormFieldFactory` — Supplies helpers for scalar, repeated, attachment, and evidence fields.
- `FormFieldFamily` — Names the four supported form field families.
- `FormFieldLayoutOptions` — Declares a field's section, row, order, and responsive layout hints.
- `FormFieldOptions` — Configures an ordinary scalar field.
- `FormFieldPath` — Represents a typed or textual path into the form value.
- `FormFieldsBuilder` — Maps application field names to field declarations.
- `FormInputAdapterCapabilitySet` — Declares which input and composition capabilities an adapter supports.
- `FormInputAdapterOptions` — Configures raw-input parsing, formatting, and composition behavior.
- `FormInputAdapterTier` — Names the adapter's supported capability tier.
- `FormRepeatedFieldOptions` — Configures identity and behavior for a repeated field.
- `FormRepeatedIdentityOptions` — Configures how repeated items receive stable identity.
- `FormRepeatedItem` — Represents one identified item exposed by a repeated field.
- `FormRepeatedResourceLocus` — Maps a repeated item to its resource-backed patch location.
- `FormValueResourceLocus` — Unites the supported resource locations for a form value.
- `FormValueResourceLocusField` — Maps a form value to a named resource field.
- `FormValueResourceLocusJsonPath` — Maps a form value to a resource JSON path.
- `FormValueResourceLocusRegion` — Maps a form value to a declared resource region.

## Handoff

Source declaration: `package/types/forms/handoff.d.ts`

- `FormHandoffPresentationArtifact` — Admits presentation state for handing work to another surface.
- `FormHandoffReport` — Aggregates current handoff presentation state and history.

## Host Facts And Bindings

Source declaration: `package/types/forms/host.d.ts`

- `FormAvailabilityHostBinding` — Maps an availability rule to a declared host capability.
- `FormFocusHostBinding` — Maps focus behavior to a host focus capability.
- `FormHostAvailabilityFact` — Carries an admitted host-provided availability fact.
- `FormHostBindings` — Declares the host capabilities consumed by the form.
- `FormHostFocusFact` — Carries an admitted host-provided focus result.
- `FormHostOnlineFact` — Carries the host's current admitted online state.
- `FormHostReport` — Aggregates host facts, required capabilities, and unavailable bindings.
- `FormHostRequiredCapability` — Names a host capability the form requires.
- `FormHostViewportFact` — Carries admitted viewport measurements from the host.
- `FormHostVisibilityFact` — Carries the host's current admitted visibility state.
- `FormOnlineHostBinding` — Maps online behavior to a host online capability.
- `FormPersistenceHostBinding` — Maps draft persistence to a host persistence capability.
- `FormViewportHostBinding` — Maps responsive behavior to a host viewport capability.
- `FormVisibilityHostBinding` — Maps visibility behavior to a host visibility capability.

## Input Capabilities

Source declaration: `package/types/forms/input_capabilities.d.ts`

- `FormInputCapabilitiesReport` — Aggregates the input capabilities available across all fields.
- `FormInputCapabilityArtifact` — Describes one field adapter's admitted and unavailable capabilities.

## Interaction

Source declaration: `package/types/forms/interaction.d.ts`

- `FormFieldInteractionArtifact` — Records one focus, blur, touch, visit, or input interaction.
- `FormFieldInteractionState` — Summarizes the retained interaction state of one field.
- `FormInteractionInputSource` — Names the host source of an interaction or input event.
- `FormInteractionReport` — Aggregates field interaction state, history, and scan costs.
- `FormSubmitIntentArtifact` — Records a user's admitted intent to submit the form.

## Layout

Source declaration: `package/types/forms/layout.d.ts`

- `FormLayoutFieldHint` — Describes one field's declared placement and responsive visibility.
- `FormLayoutReport` — Resolves declared layout hints into ordered sections, rows, and fields.
- `FormLayoutRowHint` — Describes a row's grouping, order, and responsive behavior.
- `FormLayoutSectionHint` — Describes a section's grouping, order, and responsive behavior.

## Layout Measurement

Source declaration: `package/types/forms/measurement.d.ts`

- `FormLayoutMeasurementCause` — Names why the host measured or remeasured form layout.
- `FormLayoutMeasurementDeclaration` — Configures which layout measurements the form expects.
- `FormLayoutMeasurementReport` — Aggregates admitted row measurements and pending requirements.
- `FormLayoutRowMeasurement` — Carries host-measured geometry for one form row.
- `FormLayoutSnapshotArtifact` — Records one admitted host layout snapshot.

## Media

Source declaration: `package/types/forms/media.d.ts`

- `FormMediaPresentationArtifact` — Admits host presentation state for form-related media.
- `FormMediaReport` — Aggregates media presentation state and history.

## Messages

Source declaration: `package/types/forms/messages.d.ts`

- `FormMessagePresentationArtifact` — Admits host presentation state for a form message.
- `FormMessagesReport` — Aggregates message presentation state, visibility, and history.

## Navigation

Source declaration: `package/types/forms/navigation.d.ts`

- `FormNavigationReport` — Aggregates requested form navigation and transition history.
- `FormNavigationTransitionArtifact` — Records one admitted navigation transition.

## Presentation Lifecycles

Source declaration: `package/types/forms/presentation.d.ts`

- `FormActionPresentationLanePolicy` — Configures action presentation and its settlement dependencies.
- `FormActionPresentationSettlementDependency` — Names a condition an action presentation must await before settling.
- `FormEntryBootstrapArtifact` — Explains whether the form's visible entry requirements are satisfied.
- `FormEntryBootstrapDependency` — Names a fact that may gate initial form presentation.
- `FormEntryBootstrapPolicy` — Selects which facts must be ready before the form is presented.
- `FormEntryPresentationLanePolicy` — Configures entry presentation and bootstrap requirements.
- `FormPresentationDeclaration` — Declares policies for all supported presentation lanes.
- `FormPresentationDependencyArtifact` — Records the status of one presentation dependency.
- `FormPresentationHistoryArtifact` — Unites retained lane updates and settlement records.
- `FormPresentationLane` — Names a coordinated form presentation lifecycle.
- `FormPresentationLanePolicy` — Configures timing, acknowledgement, and supersession for a lane.
- `FormPresentationLaneUpdateArtifact` — Records a host update to a presentation lane.
- `FormPresentationLifecycleArtifact` — Describes the current state and policy of one presentation lane.
- `FormPresentationReport` — Aggregates presentation lanes, acknowledgements, history, and scan costs.
- `FormPresentationScope` — Names the UI scope governed by a presentation lane.
- `FormPresentationSettlementArtifact` — Records acknowledgement, timeout, ignore, or no-op settlement.
- `FormPresentationStatus` — Names the current lifecycle status of a presentation lane.

## Replay And Restore

Source declaration: `package/types/forms/replay_restore.d.ts`

- `FormReplayRestoreArtifact` — Unites every possible resource replay or restore result.
- `FormReplayRestoreMode` — Selects exact replay or exact snapshot restoration.
- `FormReplayRestoreReplayed` — Records a successful exact resource replay.
- `FormReplayRestoreResourceArtifact` — Unites successful, unavailable, and restored resource outcomes.
- `FormReplayRestoreRestored` — Records a successful exact resource restoration.
- `FormReplayRestoreResultKind` — Names the result state of replay or restore.
- `FormReplayRestoreUnavailable` — Explains why replay or restore authority is unavailable.

## Reset And Rollback

Source declaration: `package/types/forms/reset.d.ts`

- `FormResetArtifact` — Records the result of resetting controller-local draft state.
- `FormResetEffectRejected` — Records a reset blocked by an open or rejected resource effect.
- `FormResetMode` — Selects local reset or resource-aware rollback behavior.
- `FormResetResultKind` — Names the outcome of a reset attempt.
- `FormResetRollbackArtifact` — Unites successful and unavailable rollback outcomes.
- `FormResetRollbackUnavailable` — Explains why a requested resource rollback cannot run.

## Resource Drift

Source declaration: `package/types/forms/resource_drift.d.ts`

- `FormResourceDriftArtifact` — Records a detected or resolved difference from resource authority.
- `FormResourceDriftReport` — Aggregates current resource drift, affected fields, and history.
- `FormResourceDriftStatus` — Names whether resource drift is clear, detected, or unresolved.

## Resource Source Inspection

Source declaration: `package/types/forms/resource_source.d.ts`

- `FormResourceExternalCompatibilityReport` — Reports whether an external resource line satisfies the expected contract.
- `FormResourceLifecycleReport` — Summarizes resource activity, freshness, retry, supersession, and delivery basis.
- `FormResourceMutationResponseReport` — Explains how a mutation response reconciled every targeted resource.
- `FormResourceRollbackDigest` — Summarizes exact, inverse-patch, unavailable, or inapplicable rollback support.
- `FormResourceSettlementReport` — Describes the latest resource operation's settlement and visible result.
- `FormResourceShapeReport` — Describes the resource family and patch-lowering shape behind the form.
- `FormResourceSourceReport` — Aggregates the full resource-backed source, lifecycle, history, and verification view.
- `FormResourceTransferReport` — Summarizes resource upload, processing, and download state.
- `FormResourceVisibleSelectionKind` — Names the resource value currently selected for display.
- `FormResourceVisibleSelectionProof` — States whether branch or rebase proof admits a visible value.
- `FormResourceVisibleSelectionReport` — Explains which resource value is visible and why it was selected.

## Source Authoring

Source declaration: `package/types/forms/sources.d.ts`

- `FormSourceAuthorityDiagnostics` — Explains the authority, compatibility, and retained history of a form source.
- `FormSourceDeclaration` — Defines an explicit form source and its options.
- `FormSourceFactory` — Supplies factories for signal, graph, resource, and external sources.
- `FormSourceKind` — Names the supported source authority categories.
- `FormSourceOptions` — Configures source identity, contract, migration, and equality behavior.

## Steps

Source declaration: `package/types/forms/steps.d.ts`

- `FormStepArtifact` — Describes one declared step and its current progress state.
- `FormStepContext` — Exposes safe form facts to step posture rules.
- `FormStepDeclaration` — Defines a step's fields, ordering, and posture policy.
- `FormStepDeclarationOptions` — Configures one authored form step and its behavior.
- `FormStepFactory` — Supplies helpers for declaring form steps.
- `FormStepPosture` — Names whether a step is available, active, complete, blocked, or hidden.
- `FormStepPostureArtifact` — Records the evaluated posture of one step.
- `FormStepProgress` — Summarizes the form's current location and completion through its steps.
- `FormStepsBuilder` — Maps step names to their declarations.
- `FormStepsReport` — Aggregates step posture, progress, transitions, and scan costs.

## Validation

Source declaration: `package/types/forms/validation.d.ts`

- `FormAsyncValidationLifecycleArtifact` — Records one async validation operation from start through settlement.
- `FormAsyncValidationResultKind` — Names the pending or settled outcome of async validation.
- `FormAsyncValidationTrigger` — Names the interaction or lifecycle event that starts validation.
- `FormAsyncValidationTriggerPolicy` — Configures when an async validation declaration should run.
- `FormMessageArtifact` — Describes one validation or server message attached to the form.
- `FormValidationArtifact` — Unites form messages and async validation lifecycle evidence.
- `FormValidationBuilder` — Maps validation names to their declarations.
- `FormValidationContext` — Exposes safe form reads to validation rules.
- `FormValidationFactory` — Supplies helpers for field, form, and async validation declarations.
- `FormValidationFieldReadView` — Provides read-only validation access to one field.
- `FormValidationReadView` — Provides read-only validation access to the form and its fields.
- `FormValidationReport` — Aggregates messages, async operations, validity, and scan costs.

## Verification

Source declaration: `package/types/forms/verification.d.ts`

- `FormVerificationPackage` — Packages current form digests and agreement evidence for inspection.

## How To Go Deeper

Use the [Form API Reference](forms.md) for the ordinary controller lane and the
[Forms overview](../forms/index.md) to choose a task-focused guide. When a
catalog entry is the only public explanation available, read its named
declaration file first. Then follow the corresponding implementation under
`package-src/product/forms/` and the evidence named in
`docs/metadata/public-surface-policy.json`.
