import { FormDeclarationError } from '../form_errors.js';
import { createFieldHandle } from './handles.js';

export function registerFormFields(form, fieldHandles, declarations, state, reactiveBindings) {
  const fieldsById = new Map();
  for (const declaration of declarations) {
    const handle = reactiveBindings.wrapFieldHandle(createFieldHandle(declaration, form, state));
    fieldHandles[declaration.name] = handle;
    fieldsById.set(declaration.id, handle);
  }
  Object.freeze(fieldHandles);
  Object.defineProperty(form, 'field', {
    enumerable: false,
    value(fieldId) {
      const handle = fieldsById.get(fieldId);
      if (!handle) throw new FormDeclarationError('form field is not declared', { fieldId });
      return handle;
    },
  });
}
