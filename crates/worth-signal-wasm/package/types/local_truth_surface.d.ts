import type { LocalTruthFactory } from "./local_truth/facade.js";

export * from "./local_truth/schema.js";
export * from "./local_truth/core.js";
export * from "./local_truth/facade.js";

declare module "./callable_surface.js" {
  interface CallableSignals<TPersistence> {
    readonly localTruth: LocalTruthFactory;
  }
}
