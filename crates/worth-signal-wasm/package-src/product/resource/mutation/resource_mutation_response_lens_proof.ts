const RESOURCE_MUTATION_RESPONSE_LENS_PROOF = Symbol(
  "WorthSignal.resourceMutationResponseLensProof",
);

const RESOURCE_MUTATION_RESPONSE_LENS_PROOF_VERSION =
  "resource-mutation-response-lens-proof-v1";

function createMutationResponseLensProof(options) {
  const readLensProof = options.readLensProof;
  const route = requireMutationRoute(options.route);
  const method = requireMutationMethod(options.method);
  const source = requireMutationSource(options.source, route, method);
  const declarationDigest = [
    "mutation-response-declaration",
    source,
    route,
    method,
    readLensProof.compiledLensDigest,
  ].join("|");
  const payloadDigest = [
    "mutation-response-payload",
    readLensProof.topology,
    readLensProof.itemField ?? "none",
    `aspects:${readLensProof.aspectNames.join(",")}`,
    `json:${readLensProof.jsonAspectNames.join(",")}`,
    `summaries:${readLensProof.summaryNames.join(",")}`,
    `summaryScope:${readLensProof.summaryPatchScope ?? "none"}`,
  ].join("|");
  const compiledDigest = `${declarationDigest}|${payloadDigest}`;
  return Object.freeze({
    version: RESOURCE_MUTATION_RESPONSE_LENS_PROOF_VERSION,
    source,
    route,
    method,
    topology: readLensProof.topology,
    readResponseLensSource: readLensProof.source,
    readResponseLensDigest: readLensProof.compiledLensDigest,
    declarationDigest,
    payloadDigest,
    compiledDigest,
    [RESOURCE_MUTATION_RESPONSE_LENS_PROOF]: "resourceMutationResponseLensProof",
  });
}

function requireMutationResponseLensProof(value, kind) {
  if (
    !value
    || typeof value !== "object"
    || value[RESOURCE_MUTATION_RESPONSE_LENS_PROOF] !== "resourceMutationResponseLensProof"
  ) {
    throw new TypeError(`${kind} requires a compiled mutation response lens proof`);
  }
  return value;
}

function requireMutationRoute(route) {
  if (typeof route !== "string" || route.length === 0) {
    throw new TypeError("mutation response lens proof requires a route string");
  }
  return route;
}

function requireMutationMethod(method) {
  if (method !== "POST" && method !== "PUT" && method !== "PATCH" && method !== "DELETE") {
    throw new TypeError(
      `mutation response lens proof cannot classify method "${method}"`,
    );
  }
  return method;
}

function requireMutationSource(source, route, method) {
  if (typeof source === "string" && source.length > 0) {
    return source;
  }
  return `api.url("${route}").response(...).${method.toLowerCase()}(...)`;
}

export {
  createMutationResponseLensProof,
  requireMutationResponseLensProof,
};
