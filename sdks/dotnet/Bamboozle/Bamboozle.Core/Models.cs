using System.Text.Json;
using System.Text.Json.Serialization;

namespace Bamboozle.Core;


public record ContextModel
{
    public Dictionary<string, string> QueryParams { get; set; } = [];
    public Dictionary<string, string> Headers { get; set; } = [];
    public Dictionary<string, string> RouteValues { get; set; } = [];
    public required RouteDefinition RouteModel { get; set; }
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
    public string Status { get; set; } = "200";
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

[JsonPolymorphic(TypeDiscriminatorPropertyName = "type")]
[JsonDerivedType(typeof(FixedDelayConfig), "fixed")]
[JsonDerivedType(typeof(RandomDelayConfig), "random")]
[JsonDerivedType(typeof(GaussianDelayConfig), "gaussian")]
public abstract record DelayConfig;
public sealed record FixedDelayConfig : DelayConfig
{
    [JsonPropertyName("ms")]
    public long Ms { get; set; }
}
public sealed record RandomDelayConfig : DelayConfig
{
    [JsonPropertyName("minMs")]
    public long MinMs { get; set; }
    [JsonPropertyName("maxMs")]
    public long MaxMs { get; set; }
}
public sealed record GaussianDelayConfig : DelayConfig
{
    [JsonPropertyName("meanMs")]
    public float MeanMs { get; set; }
    [JsonPropertyName("stdDevMs")]
    public float StdDevMs { get; set; }
}

public class CamelCaseEnumConverter : JsonStringEnumConverter
{
    public CamelCaseEnumConverter() : base(JsonNamingPolicy.CamelCase) {}
}

public record FaultConfig
{
    [JsonPropertyName("type")]
    public FaultKind Type { get; set; }
    [JsonPropertyName("probability")]
    public float Probability { get; set; }

    [JsonConverter(typeof(CamelCaseEnumConverter))]
    public enum FaultKind
    {
        ConnectionReset,
        EmptyResponse
    }
}