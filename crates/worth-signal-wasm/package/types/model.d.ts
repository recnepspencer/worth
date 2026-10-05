export type SignalValue =
  | null
  | boolean
  | number
  | string
  | SignalValue[]
  | { [key: string]: SignalValue };

export type AspectId = number;

export interface AspectSelectionSpec {
  aspect?: AspectId;
  aspects?: ReadonlyArray<AspectId>;
}

export interface InputOptions {
  producesAspects?: ReadonlyArray<AspectId>;
}

export interface SignalScopeSubscription {
  /** One to eight opaque path segments. */
  path: ReadonlyArray<string>;
  coverage: "Exact" | "Subtree";
}

export type RecipeReadSpec =
  | string
  | {
      id: string;
      scope?: SignalScopeSubscription;
      aspect?: AspectId;
      aspects?: ReadonlyArray<AspectId>;
    };

export interface RecipeFamilyReadScopeSpec {
  partition?: string;
  partitionFrom?: string;
  detail?: string;
  matchMode?: "WholePartition" | "PartitionAndDetail";
}

export type RecipeFamilyReadSpec =
  | {
      kind: "signal";
      id: string;
      scope?: RecipeFamilyReadScopeSpec;
      aspects?: AspectSelectionSpec;
    }
  | {
      kind: "keyed";
      familyId: string;
      scope?: RecipeFamilyReadScopeSpec;
      aspects?: AspectSelectionSpec;
    };

export type Expr =
  | { kind: "value"; value: SignalValue }
  | { kind: "read"; id: string }
  | { kind: "get"; target: Expr; field: string }
  | { kind: "at"; target: Expr; index: Expr }
  | { kind: "first"; target: Expr }
  | { kind: "last"; target: Expr }
  | { kind: "slice"; target: Expr; start: Expr; end?: Expr }
  | { kind: "join"; target: Expr; separator: Expr }
  | { kind: "flatten"; target: Expr }
  | { kind: "object"; fields: ReadonlyArray<readonly [string, Expr]> }
  | { kind: "array"; items: ReadonlyArray<Expr> }
  | { kind: "sum"; args: ReadonlyArray<Expr> }
  | { kind: "multiply"; args: ReadonlyArray<Expr> }
  | { kind: "concat"; args: ReadonlyArray<Expr> }
  | { kind: "coalesce"; args: ReadonlyArray<Expr> }
  | { kind: "length"; target: Expr }
  | { kind: "contains"; target: Expr; value: Expr }
  | { kind: "mergeObjects"; args: ReadonlyArray<Expr> }
  | { kind: "keys"; target: Expr }
  | { kind: "values"; target: Expr }
  | { kind: "hasField"; target: Expr; field: string }
  | { kind: "pick"; target: Expr; fields: ReadonlyArray<string> }
  | { kind: "omit"; target: Expr; fields: ReadonlyArray<string> }
  | { kind: "append"; target: Expr; value: Expr }
  | { kind: "abs"; target: Expr }
  | { kind: "min"; args: ReadonlyArray<Expr> }
  | { kind: "max"; args: ReadonlyArray<Expr> }
  | { kind: "sqrt"; target: Expr }
  | { kind: "sin"; target: Expr }
  | { kind: "cos"; target: Expr }
  | { kind: "floor"; target: Expr }
  | { kind: "mod"; left: Expr; right: Expr }
  | { kind: "clamp"; value: Expr; min: Expr; max: Expr }
  | { kind: "atan2"; y: Expr; x: Expr }
  | { kind: "subtract"; left: Expr; right: Expr }
  | { kind: "divide"; left: Expr; right: Expr }
  | { kind: "eq"; left: Expr; right: Expr }
  | { kind: "neq"; left: Expr; right: Expr }
  | { kind: "gt"; left: Expr; right: Expr }
  | { kind: "gte"; left: Expr; right: Expr }
  | { kind: "lt"; left: Expr; right: Expr }
  | { kind: "lte"; left: Expr; right: Expr }
  | { kind: "and"; args: ReadonlyArray<Expr> }
  | { kind: "or"; args: ReadonlyArray<Expr> }
  | { kind: "not"; arg: Expr }
  | { kind: "if"; condition: Expr; thenExpr: Expr; elseExpr: Expr };

export interface ConditionSpec {
  expr: Expr;
}

export type IdentitySpec =
  | { kind: "exact" }
  | { kind: "expr"; expr: Expr };

export interface ComputedSpec {
  reads?: ReadonlyArray<RecipeReadSpec>;
  expr: Expr;
  when?: ConditionSpec;
  identity?: IdentitySpec;
  producesAspects?: ReadonlyArray<AspectId>;
}

export interface OutputSpec {
  reads?: ReadonlyArray<RecipeReadSpec>;
  expr: Expr;
  when?: ConditionSpec;
  identity?: IdentitySpec;
  producesAspects?: ReadonlyArray<AspectId>;
}

export interface SourceSpec {
  id: string;
  initial?: SignalValue;
  producesAspects?: ReadonlyArray<AspectId>;
}

export interface KeyedSourceFamilySpec {
  familyId: string;
  initial?: SignalValue;
  producesAspects?: ReadonlyArray<AspectId>;
}

export interface RecipeSpec {
  id: string;
  reads?: ReadonlyArray<RecipeReadSpec>;
  expr: Expr;
  when?: ConditionSpec;
  identity?: IdentitySpec;
  producesAspects?: ReadonlyArray<AspectId>;
}

export interface KeyedRecipeFamilySpec {
  familyId: string;
  reads?: ReadonlyArray<RecipeFamilyReadSpec>;
  expr: Expr;
  when?: ConditionSpec;
  identity?: IdentitySpec;
  producesAspects?: ReadonlyArray<AspectId>;
}

export interface RunSummary {
  touchedNodes: number;
  nodesEvaluated: number;
  nodesRecomputed: number;
  nodesSuppressed: number;
  plansBuilt: number;
  stagesExecuted: number;
  totalNanos: string;
  evaluationNanos: string;
  commitNanos: string;
}

export interface AspectVersionSummary {
  aspect: AspectId;
  version: number;
}

export interface VersionSummary {
  id: string;
  version: number;
  aspectVersions: ReadonlyArray<AspectVersionSummary>;
}

export interface SetValueWithRegions {
  id: string;
  value: SignalValue;
  changedRegions: unknown;
  aspect?: AspectId;
  aspects?: ReadonlyArray<AspectId>;
}

export interface SetValue {
  id: string;
  value: SignalValue;
  aspect?: AspectId;
  aspects?: ReadonlyArray<AspectId>;
}

export interface KeyedSetValue {
  key: string;
  value: SignalValue;
  aspect?: AspectId;
  aspects?: ReadonlyArray<AspectId>;
}

export type TransactionOp =
  | {
      kind: "set";
      id: string;
      value: SignalValue;
      aspect?: AspectId;
      aspects?: ReadonlyArray<AspectId>;
    }
  | {
      kind: "setWithRegions";
      id: string;
      value: SignalValue;
      changedRegions: unknown;
      aspect?: AspectId;
      aspects?: ReadonlyArray<AspectId>;
    }
  | {
      kind: "setMany";
      values: ReadonlyArray<SetValue>;
    }
  | {
      kind: "setManyWithRegions";
      values: ReadonlyArray<SetValueWithRegions>;
    }
  | {
      kind: "setManyKeyed";
      familyId: string;
      values: ReadonlyArray<KeyedSetValue>;
    }
  | {
      kind: "setPackedGridRgba";
      familyId: string;
      width: number;
      height: number;
      rgba: Uint8Array;
    };

export interface WebObservationNotice {
  observerId: number;
  handleId: number;
  signalId: string;
  branchId: number;
  policy: import("./diagnostics/observation.js").ObservationPolicySummary | null;
  visited: boolean;
  recomputed: boolean;
  meaningfulChange: boolean;
  triggerMatched: boolean;
}
