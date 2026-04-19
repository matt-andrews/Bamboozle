export class BamboozleError extends Error {
  constructor(
    public readonly status: number,
    public readonly body: string,
  ) {
    super(`Bamboozle request failed with status ${status}: ${body}`);
    this.name = "BamboozleError";
  }
}
