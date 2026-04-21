import { IBamboozleAssertBuilder } from "./assert";

export interface MatchKey {
  verb: string;
  pattern: string;
}

export interface ResponseDefinition {
  status?: string;
  headers?: Record<string, string>;
  content?: string;
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
  port?: string
}

export interface AssertOptions {
  /** Boolean evalexpr expression evaluated against each recorded call */
  expression?: IBamboozleAssertBuilder;
  /** Expected call count. -1 means ≥1 when expression is set, or any count otherwise. Default: -1 */
  expect?: number;
  port?: string;
}

export interface ClientOptions {
  baseUrl?: string;
}
