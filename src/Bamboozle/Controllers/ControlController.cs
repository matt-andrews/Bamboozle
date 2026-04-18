using Bamboozle.Models;
using Bamboozle.Models.Requests;
using Bamboozle.Services;
using Bamboozle.Utilities.JsonConverters;
using Microsoft.AspNetCore.Mvc;

namespace Bamboozle.Controllers;

[ApiController]
[Route("[controller]")]
public class ControlController(RouteManagementService routeManagementService) : ControllerBase
{
    private readonly RouteManagementService _routeManagementService = routeManagementService;

    [HttpPost("routes")]
    public async Task<RouteDefinition> PostRoutes([FromBody] RouteDefinition route)
    {
        await _routeManagementService.SetRoute(route);
        return route;
    }

    [HttpPut("routes")]
    public async Task<RouteDefinition> PutRoutes([FromBody] RouteDefinition route)
    {
        await _routeManagementService.DeleteRoute(route.Match);
        await _routeManagementService.SetRoute(route);
        return route;
    }

    [HttpDelete("routes/{verb}/{pattern}")]
    public async Task<IActionResult> DeleteRoutes([FromRoute] string verb, [FromRoute] string pattern)
    {
        await _routeManagementService.DeleteRoute(new(verb, pattern));
        return Ok();
    }

    [HttpGet("routes")]
    public IAsyncEnumerable<RouteDefinition> GetRoutes()
    {
        return _routeManagementService.GetAllRoutes();
    }

    [HttpGet("routes/{verb}/{pattern}/calls")]
    public IAsyncEnumerable<ContextModel> GetRouteCalls([FromRoute] string verb, [FromRoute] string pattern)
    {
        return _routeManagementService.GetRouteCalls(new(verb, pattern));
    }

    [HttpDelete("routes/{verb}/{pattern}/calls")]
    public async Task<IActionResult> DeleteRouteCalls([FromRoute] string verb, [FromRoute] string pattern)
    {
        await _routeManagementService.DeleteRouteCalls(new(verb, pattern));
        return Ok();
    }

    [HttpPost("routes/{verb}/{pattern}/assert")]
    public async Task<IActionResult> Assert(
        [FromRoute] string verb,
        [FromRoute] string pattern,
        [FromBody] AssertRequest req,
        [FromQuery] int expect = -1)
    {
        if (!FilterParser.TryParse<ContextModel>(req.Expression, out var expression, out var error))
            return BadRequest(error);
        return await _routeManagementService.Assert(new(verb, pattern), expression, expect)
                ? Ok()
                : StatusCode(StatusCodes.Status418ImATeapot);
    }

    [HttpGet("unmatched")]
    public IAsyncEnumerable<MatchKey> GetUnmatched()
    {
        return _routeManagementService.GetUnmatchedRouteCalls();
    }

    [HttpPost("reset")]
    public async Task<IActionResult> Reset()
    {
        await _routeManagementService.Reset();
        return Ok();
    }

    [HttpGet("health")]
    public Task<IActionResult> Health()
    {
        return Task.FromResult<IActionResult>(Ok());
    }

    [HttpGet("version")]
    public Task<IActionResult> Version()
    {
        return Task.FromResult<IActionResult>(Ok(typeof(ControlController).Assembly.GetName().Version?.ToString() ?? "0.0.0"));
    }
}
