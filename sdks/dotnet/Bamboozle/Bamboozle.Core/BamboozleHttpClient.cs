using System.Net;
using System.Net.Http.Json;

namespace Bamboozle.Core;

public sealed class BamboozleHttpClient(HttpClient httpClient)
{
    private readonly HttpClient _httpClient = httpClient;
    public async Task<RouteDefinition[]> CreateRoute(RouteDefinition definition, CancellationToken cancellationToken = default)
    {
        using HttpRequestMessage req = new(HttpMethod.Post, "/control/routes")
        {
            Content = JsonContent.Create(definition)
        };
        return await SendAsync<RouteDefinition[]>(req, cancellationToken);
    }

    public async Task<RouteDefinition[]> UpdateRoute(RouteDefinition definition, CancellationToken cancellationToken = default)
    {
        using HttpRequestMessage req = new(HttpMethod.Put, "/control/routes")
        {
            Content = JsonContent.Create(definition)
        };
        return await SendAsync<RouteDefinition[]>(req, cancellationToken);
    }

    public async Task DeleteRoute(MatchKey key, CancellationToken cancellationToken = default)
    {
        using HttpRequestMessage req = new(HttpMethod.Delete, AddRouteKey("/control/routes", key));
        await SendAsync(req, cancellationToken);
    }

    public async Task<RouteDefinition[]> GetRoutes(CancellationToken cancellationToken = default)
    {
        using HttpRequestMessage req = new(HttpMethod.Get, "/control/routes");
        return await SendAsync<RouteDefinition[]>(req, cancellationToken) ?? [];
    }

    public async Task<ContextModel[]> GetRouteCalls(MatchKey key, CancellationToken cancellationToken = default)
    {
        using HttpRequestMessage req = new(HttpMethod.Get, AddRouteKey("/control/routes", key) + "/calls");
        return await SendAsync<ContextModel[]>(req, cancellationToken) ?? [];
    }

    public async Task DeleteRouteCalls(MatchKey key, CancellationToken cancellationToken = default)
    {
        using HttpRequestMessage req = new(HttpMethod.Delete, AddRouteKey("/control/routes", key) + "/calls");
        await SendAsync(req, cancellationToken);
    }

    public class AssertOptions
    {
        public int? CalledExactly { get; set; }
        public int? CalledAtLeast { get; set; }
        public int? CalledAtMost { get; set; }
        public bool NeverCalled { get; set; }

        public string GetQuery()
        {
            List<string> result = [];
            if (NeverCalled)
            {
                result.Add("never_called=true");
            }
            if (CalledExactly.HasValue)
            {
                result.Add("called_exactly=" + CalledExactly);
            }
            if (CalledAtLeast.HasValue)
            {
                result.Add("called_at_least=" + CalledAtLeast);
            }
            if (CalledAtMost.HasValue)
            {
                result.Add("called_at_most=" + CalledAtMost);
            }

            return string.Join("&", result);
        }
    }

    public async Task<bool> Assert(MatchKey key, AssertOptions options, CancellationToken cancellationToken = default)
    {
        return await Assert(key, options, expression: "", cancellationToken);
    }

    public async Task<bool> Assert(MatchKey key, AssertOptions options, BamboozleAssertBuilder builder, CancellationToken cancellationToken = default)
    {
        return await Assert(key, options, builder.ToString(), cancellationToken);
    }

    public async Task<bool> Assert(MatchKey key, AssertOptions options, string expression, CancellationToken cancellationToken = default)
    {
        string url = AddRouteKey("/control/routes", key) + "/assert?" + options.GetQuery();

        using HttpRequestMessage req = new(HttpMethod.Post, url)
        {
            Content = JsonContent.Create(new
            {
                Expression = expression
            })
        };
        using HttpResponseMessage response = await _httpClient.SendAsync(req, cancellationToken);
        if (response.StatusCode == HttpStatusCode.NotAcceptable)
        {
            return false;
        }
        response.EnsureSuccessStatusCode();
        return true;
    }

    public async Task<MatchKey[]> GetUnmatched(CancellationToken cancellationToken = default)
    {
        using HttpRequestMessage req = new(HttpMethod.Get, "/control/unmatched");
        return await SendAsync<MatchKey[]>(req, cancellationToken) ?? [];
    }

    public async Task Reset(CancellationToken cancellationToken = default)
    {
        using HttpRequestMessage req = new(HttpMethod.Post, "/control/reset");
        await SendAsync(req, cancellationToken);
    }

    public async Task Health(CancellationToken cancellationToken = default)
    {
        using HttpRequestMessage req = new(HttpMethod.Get, "/control/health");
        await SendAsync(req, cancellationToken);
    }

    public async Task<string> Version(CancellationToken cancellationToken = default)
    {
        using HttpRequestMessage req = new(HttpMethod.Get, "/control/version");
        using HttpResponseMessage response = await _httpClient.SendAsync(req, cancellationToken);
        response.EnsureSuccessStatusCode();
        return await response.Content.ReadAsStringAsync(cancellationToken);
    }

    private async Task<TResponse?> SendAsync<TResponse>(HttpRequestMessage req, CancellationToken cancellationToken)
    {
        using HttpResponseMessage response = await _httpClient.SendAsync(req, cancellationToken);
        response.EnsureSuccessStatusCode();
        return await response.Content.ReadFromJsonAsync<TResponse>(cancellationToken);
    }

    private async Task SendAsync(HttpRequestMessage req, CancellationToken cancellationToken)
    {
        using HttpResponseMessage response = await _httpClient.SendAsync(req, cancellationToken);
        response.EnsureSuccessStatusCode();
    }

    private static string AddRouteKey(string route, MatchKey key)
    {
        string escapedVerb = Uri.EscapeDataString(key.Verb);
        string escapedPattern = Uri.EscapeDataString(key.Pattern);
        return route.TrimEnd('/') + $"/{escapedVerb}/{escapedPattern}";
    }
}
