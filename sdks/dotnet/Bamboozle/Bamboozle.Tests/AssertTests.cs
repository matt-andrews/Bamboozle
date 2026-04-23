using System.Net.Http.Json;
using Bamboozle.Core;
using System.Text.Json;

namespace Bamboozle.Tests;

[Collection("Bamboozle")]
public class AssertTests : IClassFixture<BamboozleFixture>, IAsyncLifetime
{
    private readonly BamboozleFixture _fixture;

    public AssertTests(BamboozleFixture fixture)
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

    private async Task<MatchKey> SetupRoute(string verb, string pattern)
    {
        var match = new MatchKey(verb, pattern);
        var routeDef = new RouteDefinition
        {
            Match = match,
            Response = new ResponseDefinition
            {
                Status = "200",
                Content = "OK"
            }
        };
        await _fixture.Bamboozle.CreateRoute(routeDef);
        return match;
    }

    [Fact]
    public async Task Assert_CalledExactly_ShouldPassOrFailCorrectly()
    {
        var match = await SetupRoute("GET", "test-assert-exactly");

        // Make 2 calls
        await _fixture.MockClient.GetAsync("/test-assert-exactly");
        await _fixture.MockClient.GetAsync("/test-assert-exactly");

        var optionsExact = new BamboozleHttpClient.AssertOptions { CalledExactly = 2 };
        bool resultExact = await _fixture.Bamboozle.Assert(match, optionsExact);
        Assert.True(resultExact);

        var optionsWrong = new BamboozleHttpClient.AssertOptions { CalledExactly = 3 };
        bool resultWrong = await _fixture.Bamboozle.Assert(match, optionsWrong);
        Assert.False(resultWrong);
    }

    [Fact]
    public async Task Assert_CalledAtLeast_ShouldPassOrFailCorrectly()
    {
        var match = await SetupRoute("POST", "test-assert-at-least");

        await _fixture.MockClient.PostAsync("/test-assert-at-least", new StringContent(""));
        await _fixture.MockClient.PostAsync("/test-assert-at-least", new StringContent(""));

        var optionsPass = new BamboozleHttpClient.AssertOptions { CalledAtLeast = 1 };
        bool resultPass = await _fixture.Bamboozle.Assert(match, optionsPass);
        Assert.True(resultPass);

        var optionsFail = new BamboozleHttpClient.AssertOptions { CalledAtLeast = 3 };
        bool resultFail = await _fixture.Bamboozle.Assert(match, optionsFail);
        Assert.False(resultFail);
    }

    [Fact]
    public async Task Assert_NeverCalled_ShouldPassOrFailCorrectly()
    {
        var match = await SetupRoute("GET", "test-assert-never");

        var optionsNever = new BamboozleHttpClient.AssertOptions { NeverCalled = true };
        bool resultPass = await _fixture.Bamboozle.Assert(match, optionsNever);
        Assert.True(resultPass);

        await _fixture.MockClient.GetAsync("/test-assert-never");

        bool resultFail = await _fixture.Bamboozle.Assert(match, optionsNever);
        Assert.False(resultFail);
    }

    [Fact]
    public async Task GetRouteCalls_ShouldReturnRecordedContexts()
    {
        var match = await SetupRoute("POST", "test-route-calls");

        var req = new HttpRequestMessage(HttpMethod.Post, "/test-route-calls?q=search");
        req.Headers.Add("X-Custom", "TestValue");
        req.Content = JsonContent.Create(new { Id = 123, Name = "Test" });
        await _fixture.MockClient.SendAsync(req);

        var calls = await _fixture.Bamboozle.GetRouteCalls(match);

        Assert.Single(calls);
        var call = calls[0];
        Assert.Equal("search", call.QueryParams["q"]);
        Assert.Equal("TestValue", call.Headers["x-custom"]); // Headers are usually lowercased by Bamboozle
        Assert.Equal(123, call.Body.GetProperty("id").GetInt32());
    }

    [Fact]
    public async Task DeleteRouteCalls_ShouldClearRecordedContexts()
    {
        var match = await SetupRoute("GET", "test-delete-calls");

        await _fixture.MockClient.GetAsync("/test-delete-calls");
        
        var callsBefore = await _fixture.Bamboozle.GetRouteCalls(match);
        Assert.NotEmpty(callsBefore);

        await _fixture.Bamboozle.DeleteRouteCalls(match);

        var callsAfter = await _fixture.Bamboozle.GetRouteCalls(match);
        Assert.Empty(callsAfter);
    }

    [Fact]
    public async Task Assert_WithStringExpression_ShouldEvaluateCorrectly()
    {
        var match = await SetupRoute("POST", "test-string-expr");

        await _fixture.MockClient.PostAsync("/test-string-expr?filter=active", JsonContent.Create(new { user = "alice" }));
        await _fixture.MockClient.PostAsync("/test-string-expr?filter=inactive", JsonContent.Create(new { user = "bob" }));

        var options = new BamboozleHttpClient.AssertOptions { CalledAtLeast = 1 };

        // Test matching the first call
        bool resultAlice = await _fixture.Bamboozle.Assert(match, options, "query(\"filter\") == \"active\" && body(\"user\") == \"alice\"");
        Assert.True(resultAlice);

        // Test matching a non-existent combination
        bool resultNone = await _fixture.Bamboozle.Assert(match, options, "query(\"filter\") == \"active\" && body(\"user\") == \"bob\"");
        Assert.False(resultNone);
    }

    [Fact]
    public async Task Assert_WithBuilder_PropertyAccess()
    {
        var match = await SetupRoute("PUT", "test-builder-props");
        await _fixture.MockClient.PutAsync("/test-builder-props", new StringContent("raw body data"));

        var options = new BamboozleHttpClient.AssertOptions { CalledExactly = 1 };
        var builder = new BamboozleAssertBuilder()
            .With(c => c.Verb == "PUT")
            .With(c => c.Pattern == "test-builder-props")
            .With(c => c.BodyRaw == "raw body data");

        bool result = await _fixture.Bamboozle.Assert(match, options, builder);
        Assert.True(result);
    }

    [Fact]
    public async Task Assert_WithBuilder_MethodsAndStringFunctions()
    {
        var match = await SetupRoute("POST", "test-builder-methods");
        
        var req = new HttpRequestMessage(HttpMethod.Post, "/test-builder-methods?sort=desc");
        req.Headers.Add("User-Agent", "Test-Agent-1.0");
        await _fixture.MockClient.SendAsync(req);

        var options = new BamboozleHttpClient.AssertOptions { CalledExactly = 1 };
        
        var builderPass = new BamboozleAssertBuilder()
            .With(c => c.Query("sort") == "desc")
            .With(c => c.Header("user-agent").StartsWith("Test-Agent"))
            .With(c => c.Header("user-agent").Contains("1.0"));

        bool resultPass = await _fixture.Bamboozle.Assert(match, options, builderPass);
        Assert.True(resultPass);

        var builderFail = new BamboozleAssertBuilder()
            .With(c => c.Header("user-agent").EndsWith("2.0"));

        bool resultFail = await _fixture.Bamboozle.Assert(match, options, builderFail);
        Assert.False(resultFail);
    }

    [Fact]
    public async Task Assert_WithBuilder_ComplexLogicalOperators()
    {
        var match = await SetupRoute("POST", "test-builder-logic");
        await _fixture.MockClient.PostAsync("/test-builder-logic?a=1&b=2", new StringContent(""));

        var options = new BamboozleHttpClient.AssertOptions { CalledExactly = 1 };
        
        // (a == 1 || a == 3) && b == 2
        var builder = new BamboozleAssertBuilder()
            .With(c => (c.Query("a") == "1" || c.Query("a") == "3") && c.Query("b") == "2");

        bool result = await _fixture.Bamboozle.Assert(match, options, builder);
        Assert.True(result);
    }

    [Fact]
    public async Task Assert_WithBuilder_VariableEvaluation()
    {
        var match = await SetupRoute("GET", "test-builder-vars");
        await _fixture.MockClient.GetAsync("/test-builder-vars?target=dynamic_value");

        string expectedValue = "dynamic_value";
        var options = new BamboozleHttpClient.AssertOptions { CalledExactly = 1 };
        
        var builder = new BamboozleAssertBuilder()
            .With(c => c.Query("target") == expectedValue);

        bool result = await _fixture.Bamboozle.Assert(match, options, builder);
        Assert.True(result);
    }

    [Fact]
    public async Task GetUnmatched_ShouldReturnUnhandledRoutes()
    {
        await _fixture.MockClient.GetAsync("/some-unknown-route");

        var unmatched = await _fixture.Bamboozle.GetUnmatched();
        
        Assert.NotEmpty(unmatched);
        Assert.Contains(unmatched, m => m.Verb == "GET" && m.Pattern == "some-unknown-route");
    }
}
