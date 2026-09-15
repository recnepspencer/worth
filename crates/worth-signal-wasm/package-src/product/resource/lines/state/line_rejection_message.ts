// The message a rejected line reports. Load failures arrive as Error
// instances; runtime boundary refusals arrive as plain `{ code, message }`
// objects. Both carry the reason, and dropping either for a generic fallback
// would leave the line's status unable to say why it rejected.
function readLineRejectionMessage(error, fallback) {
  if (error instanceof Error) {
    return error.message;
  }
  if (
    error
    && typeof error === "object"
    && typeof error.message === "string"
    && error.message.length > 0
  ) {
    return error.message;
  }
  if (typeof error === "string" && error.length > 0) {
    return error;
  }
  return fallback;
}

export { readLineRejectionMessage };
