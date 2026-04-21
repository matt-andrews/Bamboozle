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

export interface RouteDefinition {
  match: MatchKey;
  response: ResponseDefinition;
  setState?: string;
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
  /** Expected call count. -1 means ≥1 when expression is set, or any count otherwise. Default: -1 */
  expect?: number;
}

export interface ClientOptions {
  baseUrl?: string;
}
