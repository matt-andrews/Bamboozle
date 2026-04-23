using System.Text.Json;

namespace Bamboozle.Core;

public record ContextModel
{
    public Dictionary<string, string> QueryParams { get; set; } = [];
    public Dictionary<string, string> Headers { get; set; } = [];
    public Dictionary<string, string> RouteValues { get; set; } = [];
    public required RouteDefinition RouteDefinition { get; set; }
    public JsonElement Body { get; set; }
    public string BodyRaw { get; set; } = string.Empty;
    public string State { get; set; } = string.Empty;
    public ContextModel? PreviousContext { get; set; }
}

public record MatchKey(string Verb, string Pattern);

public record RouteDefinition
{
    public required MatchKey Match { get; set; }
    public required ResponseDefinition Response { get; set; }
    public string? SetState { get; set; }
    public SimulationConfig? Simulation { get; set; }
}

public record ResponseDefinition
{
    public required string Status { get; set; }
    public Dictionary<string, string> Headers { get; set; } = [];
    public string? Content { get; set; }
    public string? ContentFile { get; set; }
    public string? BinaryFile { get; set; }
    public bool Loopback { get; set; }
}

public record SimulationConfig
{
    public DelayConfig? Delay { get; set; }
    public FaultConfig? Fault { get; set; }
}

public record DelayConfig
{
    public long Fixed { get; set; }
    public RandomRange Random { get; set; } = new(0, 0);
    public GaussianRange Gaussian { get; set; } = new(0, 0);
    public record RandomRange(long MinMs, long MaxMs);
    public record GaussianRange(float MeanMs, float StdDevMs);
}

public record FaultConfig
{
    public FaultKind Type { get; set; }
    public float Probability { get; set; }
    public enum FaultKind
    {
        ConnectionReset,
        EmptyResponse
    }
}