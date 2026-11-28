import { buildPatchPlan, dirtyFieldRecords } from "./patching/patch_planning.js";
import { clearedFieldIds, omittedFieldIds } from "./availability/artifacts.js";
import { evaluateAvailability } from "./availability/execution.js";
import { evaluateAdmission } from "./admission/execution.js";
import { validateForm } from "./validation/execution.js";
import { visibleMessages } from "./validation/artifacts.js";
import { evaluateSteps } from "./steps/artifacts.js";
import { planActions } from "./actions/planning.js";

export function createDerivedReportBindings({
  formRef,
  sourceDeclaration,
  syncSourceCompatibility,
  authoritativeSource,
  fieldDeclarations,
  rawInputs,
  parseFailures,
  asyncValidations,
  validationDeclarations,
  availabilityDeclarations,
  admissionDeclarations,
  stepDeclarations,
  actionDeclarations,
}) {
  return Object.freeze({
    dirty() {
      syncSourceCompatibility(authoritativeSource());
      const availability = formRef().availability();
      const dirtyFields = dirtyFieldRecords(fieldDeclarations, formRef(), {
        omittedFields: omittedFieldIds(availability),
        clearedFields: clearedFieldIds(availability),
      });
      return Object.freeze({
        isDirty: dirtyFields.fields.length > 0,
        semanticDirty: dirtyFields.fields.length > 0,
        fields: dirtyFields.fields,
        equality: dirtyFields.equality,
        breadth: dirtyFields.breadth,
      });
    },
    patchPlan() {
      syncSourceCompatibility(authoritativeSource());
      const availability = formRef().availability();
      return buildPatchPlan(fieldDeclarations, formRef(), rawInputs, {
        omittedFields: omittedFieldIds(availability),
        clearedFields: clearedFieldIds(availability),
      });
    },
    validation() {
      syncSourceCompatibility(authoritativeSource());
      return validateForm(
        fieldDeclarations,
        validationDeclarations,
        formRef(),
        parseFailures,
        asyncValidations.artifacts(),
      );
    },
    availability() {
      syncSourceCompatibility(authoritativeSource());
      return evaluateAvailability(availabilityDeclarations, formRef());
    },
    admission() {
      syncSourceCompatibility(authoritativeSource());
      return evaluateAdmission(admissionDeclarations, formRef(), fieldDeclarations);
    },
    visibleMessages() {
      return Object.freeze([
        ...visibleMessages(formRef().validation()),
      ]);
    },
    steps() {
      syncSourceCompatibility(authoritativeSource());
      return evaluateSteps(stepDeclarations, formRef());
    },
    actions() {
      syncSourceCompatibility(authoritativeSource());
      return planActions(actionDeclarations, formRef(), fieldDeclarations, sourceDeclaration);
    },
  });
}
