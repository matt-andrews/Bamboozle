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
}

export interface ContextModel {
  query_params: Record<string, string>;
  headers: Record<string, string>;
  route_values: Record<string, string>;
  route_model: RouteDefinition;
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
