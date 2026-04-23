import type { IBamboozleAssertBuilder } from "./assert.js";

export interface MatchKey {
  verb: string;
  pattern: string;
}

export interface ResponseDefinition {
  status?: string;
  headers?: Record<string, string>;
  content?: string;
  contentFile?: string;
  binaryFile?: string;
  loopback?: boolean;
}

export type DelayConfig =
  | { type: 'fixed'; ms: number }
  | { type: 'random'; minMs: number; maxMs: number }
  | { type: 'gaussian'; meanMs: number; stdDevMs: number };

export interface FaultConfig {
  type: 'connectionReset' | 'emptyResponse';
  /** 0.0–1.0. Defaults to 1.0 (always trigger). */
  probability?: number;
}

export interface SimulationConfig {
  delay?: DelayConfig;
  fault?: FaultConfig;
}

export interface RouteDefinition {
  match: MatchKey;
  response: ResponseDefinition;
  setState?: string;
  simulation?: SimulationConfig;
}

export interface ContextModel {
  queryParams: Record<string, string>;
  headers: Record<string, string>;
  routeValues: Record<string, string>;
  routeModel: RouteDefinition;
}

export interface AssertOptions {
  /** Boolean evalexpr expression evaluated against each recorded call */
  expression?: IBamboozleAssertBuilder;
  /** Assert the filtered call count equals exactly n */
  calledExactly?: number;
  /** Assert the filtered call count is at least n */
  calledAtLeast?: number;
  /** Assert the filtered call count is at most n */
  calledAtMost?: number;
  /** Assert the route was never called */
  neverCalled?: boolean;
}

export interface ClientOptions {
  baseUrl?: string;
}
