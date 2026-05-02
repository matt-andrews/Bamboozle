import { BamboozleError } from "./errors.js";
import type {
  AssertOptions,
  ClientOptions,
  ContextModel,
  MatchKey,
  RouteDefinition,
} from "./types.js";

export class BamboozleClient {
  readonly #baseUrl: string;

  constructor(options: ClientOptions = {}) {
    this.#baseUrl = (options.baseUrl ?? "http://localhost:9090").replace(/\/$/, "");
  }

  async addRoute(route: RouteDefinition): Promise<RouteDefinition[]> {
    return this.#request("POST", "/control/routes", route);
  }

  async upsertRoute(route: RouteDefinition): Promise<RouteDefinition[]> {
    return this.#request("PUT", "/control/routes", route);
  }

  async listRoutes(): Promise<RouteDefinition[]> {
    return this.#request("GET", "/control/routes");
  }

  async deleteRoute(verb: string, pattern: string): Promise<void> {
    await this.#request("DELETE", this.#routePath(verb, pattern));
  }

  async getCalls(verb: string, pattern: string): Promise<ContextModel[]> {
    return this.#request("GET", `${this.#routePath(verb, pattern)}/calls`);
  }

  async clearCalls(verb: string, pattern: string): Promise<void> {
    await this.#request("DELETE", `${this.#routePath(verb, pattern)}/calls`);
  }

  async assert(verb: string, pattern: string, options: AssertOptions = {}): Promise<boolean> {
    const url = new URL(`${this.#baseUrl}${this.#routePath(verb, pattern)}/assert`);
    if (options.neverCalled === true) {
      url.searchParams.set("never_called", "true");
    }
    if (options.calledExactly !== undefined) {
      url.searchParams.set("called_exactly", String(options.calledExactly));
    }
    if (options.calledAtLeast !== undefined) {
      url.searchParams.set("called_at_least", String(options.calledAtLeast));
    }
    if (options.calledAtMost !== undefined) {
      url.searchParams.set("called_at_most", String(options.calledAtMost));
    }
    const body = options.expression !== undefined ? { expression: options.expression.build() } : { expression: "" };

    const init: RequestInit = { method: "POST" };
    if (Object.keys(body).length > 0) {
      init.body = JSON.stringify(body);
      init.headers = { "Content-Type": "application/json" };
    }

    const res = await this.#safeFetch(url.toString(), init);
    if (res.status === 406) return false;
    if (!res.ok) throw new BamboozleError(res.status, await res.text());
    return true;
  }

  async getUnmatched(): Promise<MatchKey[]> {
    return this.#request("GET", "/control/unmatched");
  }

  async reset(): Promise<void> {
    await this.#request("POST", "/control/reset");
  }

  async health(): Promise<void> {
    await this.#request("GET", "/control/health");
  }

  async version(): Promise<string> {
    return this.#request("GET", "/control/version");
  }

  #routePath(verb: string, pattern: string): string {
    return `/control/routes/${encodeURIComponent(verb)}/${encodeURIComponent(pattern)}`;
  }

  async #request<T>(method: string, path: string, body?: unknown): Promise<T> {
    const url = `${this.#baseUrl}${path}`;
    const init: RequestInit = { method };

    if (body !== undefined && Object.keys(body as object).length > 0) {
      init.body = JSON.stringify(body);
      init.headers = { "Content-Type": "application/json" };
    }

    const res = await this.#safeFetch(url, init);

    if (!res.ok) {
      const text = await res.text();
      throw new BamboozleError(res.status, text);
    }

    const text = await res.text();
    if (!text) return undefined as T;

    try {
      return JSON.parse(text) as T;
    } catch {
      return text as T;
    }
  }

  async #safeFetch(url: string, init: RequestInit): Promise<Response> {
    try {
      return await fetch(url, init);
    } catch (err) {
      const cause = err instanceof Error ? err : new Error(String(err));
      throw new Error(`Bamboozle request to ${init.method} ${url} failed: ${cause.message}`, { cause });
    }
  }
}
