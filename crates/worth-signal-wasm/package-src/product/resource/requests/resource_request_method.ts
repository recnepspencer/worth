const RESOURCE_REQUEST_METHODS = Object.freeze({
  get: "GET",
  post: "POST",
  put: "PUT",
  patch: "PATCH",
  delete: "DELETE",
});

function requireResourceRequestMethod(method, family) {
  if (
    method !== RESOURCE_REQUEST_METHODS.get
    && method !== RESOURCE_REQUEST_METHODS.post
    && method !== RESOURCE_REQUEST_METHODS.patch
    && method !== RESOURCE_REQUEST_METHODS.put
    && method !== RESOURCE_REQUEST_METHODS.delete
  ) {
    throw new TypeError(
      `${family} resource request method must be "GET", "POST", "PUT", "PATCH", or "DELETE"`,
    );
  }
  return method;
}

export {
  RESOURCE_REQUEST_METHODS,
  requireResourceRequestMethod,
};
