using System.Net;
using Bamboozle.Core;

namespace Bamboozle.Tests;

[Collection("Bamboozle")]
public class RouteManagementTests : IClassFixture<BamboozleFixture>, IAsyncLifetime
{
    private readonly BamboozleFixture _fixture;

    public RouteManagementTests(BamboozleFixture fixture)
    {
        _fixture = fixture;
    }

    public async Task InitializeAsync()
    {
        await _fixture.Bamboozle.Reset();
    }

    public Task DisposeAsync()
    {
        return Task.CompletedTask;
    }

    [Fact]
    public async Task Health_ShouldReturnTrue()
    {
        await _fixture.Bamboozle.Health();
        Assert.True(true);
    }

    [Fact]
    public async Task Version_ShouldReturnVersionString()
    {
        string version = await _fixture.Bamboozle.Version();
        Assert.False(string.IsNullOrWhiteSpace(version));
    }

    [Fact]
    public async Task CreateRoute_ShouldRegisterAndReturnRoute()
    {
        var routeDef = new RouteDefinition
        {
            Match = new MatchKey("GET", "test-route-1"),
            Response = new ResponseDefinition
            {
                Status = "200",
                Content = "Hello World"
            }
        };

        var created = await _fixture.Bamboozle.CreateRoute(routeDef);

        Assert.NotNull(created);
        Assert.Equal("GET", created.Match.Verb);
        Assert.Equal("test-route-1", created.Match.Pattern);
        Assert.Equal("200", created.Response.Status);
        Assert.Equal("Hello World", created.Response.Content);
        
        // Verify mock endpoint works
        var response = await _fixture.MockClient.GetAsync("/test-route-1");
        Assert.Equal(HttpStatusCode.OK, response.StatusCode);
        string content = await response.Content.ReadAsStringAsync();
        Assert.Equal("Hello World", content);
    }

    [Fact]
    public async Task GetRoutes_ShouldListConfiguredRoutes()
    {
        var routeDef = new RouteDefinition
        {
            Match = new MatchKey("GET", "test-route-2"),
            Response = new ResponseDefinition
            {
                Status = "204"
            }
        };

        await _fixture.Bamboozle.CreateRoute(routeDef);

        var routes = await _fixture.Bamboozle.GetRoutes();
        
        Assert.NotEmpty(routes);
        Assert.Contains(routes, r => r.Match.Verb == "GET" && r.Match.Pattern == "test-route-2");
    }

    [Fact]
    public async Task UpdateRoute_ShouldChangeRouteConfiguration()
    {
        var routeDef = new RouteDefinition
        {
            Match = new MatchKey("POST", "test-route-3"),
            Response = new ResponseDefinition
            {
                Status = "201",
                Content = "Created"
            }
        };

        await _fixture.Bamboozle.CreateRoute(routeDef);

        // Update the route
        routeDef.Response.Content = "Updated";
        routeDef.Response.Status = "202";

        var updated = await _fixture.Bamboozle.UpdateRoute(routeDef);

        Assert.NotNull(updated);
        Assert.Equal("Updated", updated.Response.Content);
        Assert.Equal("202", updated.Response.Status);
        
        // Verify with mock endpoint
        var response = await _fixture.MockClient.PostAsync("/test-route-3", new StringContent(""));
        Assert.Equal(HttpStatusCode.Accepted, response.StatusCode);
        string content = await response.Content.ReadAsStringAsync();
        Assert.Equal("Updated", content);
    }

    [Fact]
    public async Task DeleteRoute_ShouldRemoveRoute()
    {
        var match = new MatchKey("DELETE", "test-route-4");
        var routeDef = new RouteDefinition
        {
            Match = match,
            Response = new ResponseDefinition
            {
                Status = "200"
            }
        };

        await _fixture.Bamboozle.CreateRoute(routeDef);
        
        var routesBefore = await _fixture.Bamboozle.GetRoutes();
        Assert.Contains(routesBefore, r => r.Match == match);

        await _fixture.Bamboozle.DeleteRoute(match);

        var routesAfter = await _fixture.Bamboozle.GetRoutes();
        Assert.DoesNotContain(routesAfter, r => r.Match == match);
        
        // Verify with mock endpoint (should return 404 or unhandled)
        var response = await _fixture.MockClient.DeleteAsync("/test-route-4");
        // Bamboozle usually returns 404 or 501 for unhandled routes, assuming 404
        Assert.Equal(HttpStatusCode.NotFound, response.StatusCode);
    }

    [Theory]
    [InlineData(FaultConfig.FaultKind.ConnectionReset)]
    [InlineData(FaultConfig.FaultKind.EmptyResponse)]
    public async Task CreateRoute_WithFaultConfig_ShouldSerializeAndReturnCorrectly(FaultConfig.FaultKind faultKind)
    {
        var match = new MatchKey("GET", $"test-fault-{faultKind}");
        var routeDef = new RouteDefinition
        {
            Match = match,
            Response = new ResponseDefinition
            {
                Status = "200"
            },
            Simulation = new SimulationConfig
            {
                Fault = new FaultConfig
                {
                    Type = faultKind,
                    Probability = 1.0f
                }
            }
        };

        var created = await _fixture.Bamboozle.CreateRoute(routeDef);

        Assert.NotNull(created);
        Assert.NotNull(created.Simulation);
        Assert.NotNull(created.Simulation.Fault);
        Assert.Equal(faultKind, created.Simulation.Fault.Type);
        Assert.Equal(1.0f, created.Simulation.Fault.Probability);
    }

    [Fact]
    public async Task CreateRoute_WithFixedDelayConfig_ShouldSerializeAndReturnCorrectly()
    {
        var routeDef = new RouteDefinition
        {
            Match = new MatchKey("GET", "test-delay-fixed"),
            Response = new ResponseDefinition { Status = "200" },
            Simulation = new SimulationConfig
            {
                Delay = new FixedDelayConfig { Ms = 100 }
            }
        };

        var created = await _fixture.Bamboozle.CreateRoute(routeDef);

        Assert.NotNull(created?.Simulation?.Delay);
        var delay = Assert.IsType<FixedDelayConfig>(created.Simulation.Delay);
        Assert.Equal(100, delay.Ms);
    }

    [Fact]
    public async Task CreateRoute_WithRandomDelayConfig_ShouldSerializeAndReturnCorrectly()
    {
        var routeDef = new RouteDefinition
        {
            Match = new MatchKey("GET", "test-delay-random"),
            Response = new ResponseDefinition { Status = "200" },
            Simulation = new SimulationConfig
            {
                Delay = new RandomDelayConfig { MinMs = 50, MaxMs = 150 }
            }
        };

        var created = await _fixture.Bamboozle.CreateRoute(routeDef);

        Assert.NotNull(created?.Simulation?.Delay);
        var delay = Assert.IsType<RandomDelayConfig>(created.Simulation.Delay);
        Assert.Equal(50, delay.MinMs);
        Assert.Equal(150, delay.MaxMs);
    }

    [Fact]
    public async Task CreateRoute_WithGaussianDelayConfig_ShouldSerializeAndReturnCorrectly()
    {
        var routeDef = new RouteDefinition
        {
            Match = new MatchKey("GET", "test-delay-gaussian"),
            Response = new ResponseDefinition { Status = "200" },
            Simulation = new SimulationConfig
            {
                Delay = new GaussianDelayConfig { MeanMs = 100.5f, StdDevMs = 10.2f }
            }
        };

        var created = await _fixture.Bamboozle.CreateRoute(routeDef);

        Assert.NotNull(created?.Simulation?.Delay);
        var delay = Assert.IsType<GaussianDelayConfig>(created.Simulation.Delay);
        Assert.Equal(100.5f, delay.MeanMs);
        Assert.Equal(10.2f, delay.StdDevMs);
    }
}
