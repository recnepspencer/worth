// Resource line values are JSON values. A loaded result is canonicalized to
// the JSON it would serialize to, once, before it is committed: `toJSON` is
// honored exactly as JSON.stringify honors it (a `Date` becomes its ISO
// string), `undefined` members are omitted, non-finite numbers become
// `null`, and a value JSON would misrepresent (a `Map` or `Set` serialized
// as `{}`, a typed array, a boxed primitive, a function) is refused with the
// reason instead of being stored as something it is not.
//
// Cost: O(size of the value) per commit, the same order as storing it.
function canonicalizeLineValue(value, rootPath = "$value") {
  return canonicalize(value, new Set(), rootPath, true);
}

// `honorToJSON` is false for the value a `toJSON()` call returned, mirroring
// JSON.stringify: `toJSON` is consulted once per value, never on its result.
function canonicalize(value, seen, path, honorToJSON) {
  if (typeof value === "bigint" || typeof value === "function" || typeof value === "symbol") {
    throw new TypeError(
      `resource line value cannot represent ${typeof value} at ${path}: JSON has no form for it`,
    );
  }
  if (typeof value === "number") {
    return Number.isFinite(value) ? value : null;
  }
  if (!value || typeof value !== "object") {
    return value;
  }
  if (honorToJSON && typeof value.toJSON === "function") {
    return canonicalize(value.toJSON(), seen, `${path}.toJSON()`, false);
  }
  if (Array.isArray(value)) {
    return canonicalizeArray(value, seen, path);
  }
  const unrepresentable = readJsonUnrepresentableTypeName(value);
  if (unrepresentable !== null) {
    throw new TypeError(
      `resource line value cannot represent ${unrepresentable} at ${path}: `
      + "JSON serializes it without its contents, so the line could not hold it faithfully",
    );
  }
  return canonicalizeObject(value, seen, path);
}

// Object types JSON.stringify serializes as `{}` (or as a boxed primitive's
// wrapper) regardless of contents. `Date` is not here: Date#toJSON is honored
// above.
function readJsonUnrepresentableTypeName(value) {
  if (value instanceof Map) return "Map";
  if (value instanceof Set) return "Set";
  if (value instanceof WeakMap) return "WeakMap";
  if (value instanceof WeakSet) return "WeakSet";
  if (value instanceof Promise) return "Promise";
  if (value instanceof RegExp) return "RegExp";
  if (value instanceof Error) return value.constructor?.name ?? "Error";
  if (value instanceof ArrayBuffer) return "ArrayBuffer";
  if (typeof SharedArrayBuffer !== "undefined" && value instanceof SharedArrayBuffer) {
    return "SharedArrayBuffer";
  }
  if (ArrayBuffer.isView(value)) return value.constructor?.name ?? "ArrayBufferView";
  if (value instanceof Number || value instanceof String || value instanceof Boolean) {
    return `boxed ${typeof value.valueOf()}`;
  }
  return null;
}

function canonicalizeArray(value, seen, path) {
  if (seen.has(value)) {
    throw new TypeError(`resource line value cannot represent a cyclic array at ${path}`);
  }
  seen.add(value);
  try {
    const canonicalArray = [];
    for (let index = 0; index < value.length; index += 1) {
      const descriptor = Object.getOwnPropertyDescriptor(value, String(index));
      if (descriptor === undefined) {
        throw new TypeError(
          `resource line value cannot represent a sparse array slot at ${path}[${index}]`,
        );
      }
      if ("get" in descriptor || "set" in descriptor) {
        throw new TypeError(
          `resource line value cannot inspect accessor-backed array slot at ${path}[${index}]`,
        );
      }
      // JSON.stringify writes `null` for an undefined array element.
      canonicalArray.push(
        descriptor.value === undefined
          ? null
          : canonicalize(descriptor.value, seen, `${path}[${index}]`, true),
      );
    }
    return canonicalArray;
  } finally {
    seen.delete(value);
  }
}

function canonicalizeObject(value, seen, path) {
  if (seen.has(value)) {
    throw new TypeError(`resource line value cannot represent a cyclic object at ${path}`);
  }
  seen.add(value);
  const result = {};
  try {
    for (const key of Object.keys(value).sort()) {
      const descriptor = Object.getOwnPropertyDescriptor(value, key);
      if (descriptor === undefined) {
        continue;
      }
      if ("get" in descriptor || "set" in descriptor) {
        throw new TypeError(
          `resource line value cannot inspect accessor-backed property "${key}" at ${path}`,
        );
      }
      if (key === "toJSON" && typeof descriptor.value === "function") {
        // The serializer itself, already honored (or deliberately bypassed
        // when it returned this object); JSON.stringify omits it too.
        continue;
      }
      if (descriptor.value === undefined) {
        // JSON.stringify omits undefined members.
        continue;
      }
      result[key] = canonicalize(descriptor.value, seen, `${path}.${key}`, true);
    }
    return result;
  } finally {
    seen.delete(value);
  }
}

export { canonicalizeLineValue };
