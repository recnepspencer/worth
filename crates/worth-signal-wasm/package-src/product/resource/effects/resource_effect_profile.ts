import { createResourceEffectCloseoutMatrix } from "./resource_effect_closeout_matrix.js";

const RESOURCE_EFFECT_PROFILE_BRAND = Symbol(
  "WorthSignal.resourceEffectProfile",
);

const PROFILE_DEFINITIONS = Object.freeze({
  branchNative: Object.freeze({
    name: "branchNative",
    optimism: "branchSpeculative",
    confirmation: "serverCanonical",
    rollback: "branchRestoreOrInverse",
    rebase: "unavailable",
    preimage: "compactInverse",
  }),
  serverCanonical: Object.freeze({
    name: "serverCanonical",
    optimism: "branchSpeculative",
    confirmation: "serverCanonical",
    rollback: "branchRestoreOrInverse",
    rebase: "unavailable",
    preimage: "compactInverse",
  }),
  pessimistic: Object.freeze({
    name: "pessimistic",
    optimism: "none",
    confirmation: "serverCanonical",
    rollback: "unavailable",
    rebase: "unavailable",
    preimage: "none",
  }),
  deliveryAuthoritative: Object.freeze({
    name: "deliveryAuthoritative",
    optimism: "none",
    confirmation: "acceptedPendingDelivery",
    rollback: "unavailable",
    rebase: "unavailable",
    preimage: "none",
  }),
  nonReversible: Object.freeze({
    name: "nonReversible",
    optimism: "none",
    confirmation: "serverCanonical",
    rollback: "unavailable",
    rebase: "unavailable",
    preimage: "none",
  }),
  sensitive: Object.freeze({
    name: "sensitive",
    optimism: "branchSpeculative",
    confirmation: "serverCanonical",
    rollback: "branchRestore",
    rebase: "unavailable",
    preimage: "digestOnly",
  }),
});

const RESOURCE_EFFECT_PROFILES = Object.freeze(
  Object.fromEntries(
    Object.entries(PROFILE_DEFINITIONS).map(([key, definition]) => [
      key,
      createResourceEffectProfile(definition),
    ]),
  ),
);

const resourceEffects = Object.freeze({
  branchNative() {
    return RESOURCE_EFFECT_PROFILES.branchNative;
  },
  serverCanonical() {
    return RESOURCE_EFFECT_PROFILES.serverCanonical;
  },
  pessimistic() {
    return RESOURCE_EFFECT_PROFILES.pessimistic;
  },
  deliveryAuthoritative() {
    return RESOURCE_EFFECT_PROFILES.deliveryAuthoritative;
  },
  nonReversible() {
    return RESOURCE_EFFECT_PROFILES.nonReversible;
  },
  sensitive() {
    return RESOURCE_EFFECT_PROFILES.sensitive;
  },
  custom(options) {
    return createResourceEffectProfile(requireCustomProfileOptions(options));
  },
  closeoutMatrix(profile) {
    return createResourceEffectCloseoutMatrix(
      requireResourceEffectProfile(
        profile,
        "resource.effects.closeoutMatrix(...)",
      ),
    );
  },
});

function createResourceEffectProfile(options) {
  return Object.freeze({
    ...options,
    [RESOURCE_EFFECT_PROFILE_BRAND]: "resourceEffectProfile",
  });
}

function requireCustomProfileOptions(options) {
  if (!options || typeof options !== "object" || Array.isArray(options)) {
    throw new TypeError("resource.effects.custom(...) requires an options object");
  }
  const name = requireProfileName(options.name);
  const optimism = requireProfileField("optimism", options.optimism, [
    "branchSpeculative",
    "none",
  ]);
  const confirmation = requireProfileField("confirmation", options.confirmation, [
    "exact",
    "serverCanonical",
    "acceptedPendingDelivery",
  ]);
  const rollback = requireProfileField("rollback", options.rollback, [
    "branchRestore",
    "branchRestoreOrInverse",
    "unavailable",
  ]);
  const rebase = requireProfileField("rebase", options.rebase, [
    "unavailable",
  ]);
  const preimage = requireProfileField("preimage", options.preimage, [
    "none",
    "compactInverse",
    "digestOnly",
    "retainedFragment",
  ]);
  requireProfileCompatibility(optimism, rollback, preimage);
  return { name, optimism, confirmation, rollback, rebase, preimage };
}

function requireProfileName(name) {
  if (typeof name !== "string" || name.length === 0) {
    throw new TypeError(
      "resource.effects.custom(...) requires a non-empty name",
    );
  }
  return name;
}

function requireProfileField(field, value, allowed) {
  if (!allowed.includes(value)) {
    throw new TypeError(
      `resource.effects.custom(...) ${field} must be one of: ${allowed.join(", ")}`,
    );
  }
  return value;
}

function requireProfileCompatibility(optimism, rollback, preimage) {
  if (optimism === "branchSpeculative" && rollback === "unavailable") {
    throw new TypeError(
      "resource.effects.custom(...) cannot enable branch speculation with unavailable rollback",
    );
  }
  if (rollback === "unavailable" && preimage !== "none") {
    throw new TypeError(
      "resource.effects.custom(...) cannot retain preimage material when rollback is unavailable",
    );
  }
}

function requireResourceEffectProfile(value, source) {
  if (!isResourceEffectProfile(value)) {
    throw new TypeError(
      `${source} requires a profile created with resource.effects.*()`,
    );
  }
  return value;
}

function isResourceEffectProfile(value) {
  return (
    Boolean(value) &&
    value[RESOURCE_EFFECT_PROFILE_BRAND] === "resourceEffectProfile"
  );
}

export {
  isResourceEffectProfile,
  requireResourceEffectProfile,
  resourceEffects,
};
