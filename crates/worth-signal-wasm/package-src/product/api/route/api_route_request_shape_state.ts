import { requireResourceRequestMethod } from "../../resource/requests/resource_request_method.js";
import { requireResourceEffectProfile } from "../../resource/effects/resource_effect_profile.js";

function createApiRouteRequestShapeState() {
  return Object.freeze({
    method: undefined,
    bodyDeclared: false,
    headers: undefined,
    effects: undefined,
  });
}

function withApiRouteVerb(state, route, method) {
  return Object.freeze({
    ...state,
    method: requireApiRouteVerb(route, method),
  });
}

function withApiRouteBody(state, route) {
  if (state.bodyDeclared) {
    throw new TypeError(
      `api.url("${route}").body(...) is already declared in this route lane`,
    );
  }
  return Object.freeze({
    ...state,
    bodyDeclared: true,
  });
}

function withApiRouteHeaders(state, route, headers) {
  if (state.headers !== undefined) {
    throw new TypeError(
      `api.url("${route}").headers(...) is already declared in this route lane`,
    );
  }
  if (
    typeof headers !== "function"
    && (
      !headers
      || typeof headers !== "object"
      || Array.isArray(headers)
    )
  ) {
    throw new TypeError(
      `api.url("${route}").headers(...) requires a plain object or params => plain object`,
    );
  }
  return Object.freeze({
    ...state,
    headers,
  });
}

function withApiRouteEffects(state, route, effects) {
  if (state.effects !== undefined) {
    throw new TypeError(
      `api.url("${route}").effects(...) is already declared in this route lane`,
    );
  }
  if (typeof effects !== "function") {
    requireResourceEffectProfile(effects, `api.url("${route}").effects(...)`);
  }
  return Object.freeze({
    ...state,
    effects,
  });
}

function hasAdvancedApiRouteRequestShape(state) {
  return state.method !== undefined || state.bodyDeclared;
}

function requireApiRouteVerb(route, method) {
  try {
    return requireResourceRequestMethod(method, 'api.url(...)');
  } catch {
    throw new TypeError(
      `api.url("${route}").verb(...) must be "GET", "POST", "PUT", "PATCH", or "DELETE"`,
    );
  }
}

export {
  createApiRouteRequestShapeState,
  hasAdvancedApiRouteRequestShape,
  withApiRouteBody,
  withApiRouteEffects,
  withApiRouteHeaders,
  withApiRouteVerb,
};
