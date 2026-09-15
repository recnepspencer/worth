/**
 * Whether two signal snapshots carry the same value.
 *
 * React's `useSyncExternalStore` requires `getSnapshot` to return the same
 * reference while nothing changed. A fresh root read of an object-valued
 * signal can hand back a new, structurally equal object (the compatibility
 * deployment deserializes on every read), so the store compares the fresh
 * value with its cached snapshot and keeps the cached reference when they
 * match. Signal values are JSON values (primitives, arrays, plain objects);
 * anything else is never treated as equal, so a foreign value is adopted as a
 * change rather than mistaken for the cache. O(size of the smaller value).
 */
export function sameSignalSnapshot(cached: unknown, fresh: unknown): boolean {
  if (Object.is(cached, fresh)) {
    return true;
  }
  if (Array.isArray(cached)) {
    if (!Array.isArray(fresh) || cached.length !== fresh.length) {
      return false;
    }
    for (let index = 0; index < cached.length; index += 1) {
      if (!sameSignalSnapshot(cached[index], fresh[index])) {
        return false;
      }
    }
    return true;
  }
  if (!isPlainObject(cached) || !isPlainObject(fresh)) {
    return false;
  }
  const cachedKeys = Object.keys(cached);
  if (cachedKeys.length !== Object.keys(fresh).length) {
    return false;
  }
  for (const key of cachedKeys) {
    if (!Object.prototype.hasOwnProperty.call(fresh, key)) {
      return false;
    }
    if (!sameSignalSnapshot(cached[key], fresh[key])) {
      return false;
    }
  }
  return true;
}

function isPlainObject(value: unknown): value is Record<string, unknown> {
  if (value === null || typeof value !== "object" || Array.isArray(value)) {
    return false;
  }
  const prototype = Object.getPrototypeOf(value);
  return prototype === Object.prototype || prototype === null;
}
