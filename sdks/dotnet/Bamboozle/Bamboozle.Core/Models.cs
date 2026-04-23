using System.Text.Json;
using System.Text.Json.Serialization;

namespace Bamboozle.Core;


public record ContextModel
{
    [JsonPropertyName("queryParams")]
    public Dictionary<string, string> QueryParams { get; set; } = [];
    [JsonPropertyName("headers")]
    public Dictionary<string, string> Headers { get; set; } = [];
    [JsonPropertyName("routeValues")]
    public Dictionary<string, string> RouteValues { get; set; } = [];
    [JsonPropertyName("routeModel")]
    public required RouteDefinition RouteDefinition { get; set; }
    [JsonPropertyName("body")]
    public JsonElement Body { get; set; }
    [JsonPropertyName("bodyRaw")]
    public string BodyRaw { get; set; } = string.Empty;
    [JsonPropertyName("state")]
    public string State { get; set; } = string.Empty;
    [JsonPropertyName("previousContext")]
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