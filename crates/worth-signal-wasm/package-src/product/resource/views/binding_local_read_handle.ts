/** Keeps product identity while reading current binding truth and its inputs. */
export function createBindingLocalResourceReadHandle(underlying, reads) {
  let released = false;
  function requireActive() {
    if (released) throw new TypeError('resource line view cannot be used after line.free()');
  }
  const current = () => { requireActive(); return reads.current(); };
  const peek = () => { requireActive(); return reads.peek(); };
  function release() {
    if (released || !reads.release) return;
    released = true;
    reads.release();
  }
  const handle = function bindingLocalResourceReadHandle() {
    return current();
  };
  Object.defineProperties(handle, {
    id: { enumerable: true, get: () => underlying.id },
    debugName: { enumerable: true, get: () => underlying.debugName ?? null },
    get: { value: current },
    peek: { value: peek },
    free: { value: release },
    [Symbol.dispose]: { value: release },
  });
  for (const symbol of Object.getOwnPropertySymbols(underlying)) {
    if (symbol !== Symbol.dispose) {
      Object.defineProperty(handle, symbol, { get: () => underlying[symbol] });
    }
  }
  return handle;
}
