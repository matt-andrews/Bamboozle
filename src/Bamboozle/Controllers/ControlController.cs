using Bamboozle.Models;
using Bamboozle.Services;
using Microsoft.AspNetCore.Mvc;
using System.Text.Json;

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
    public Task<IEnumerable<ContextModel>> GetRouteCalls([FromRoute] string verb, [FromRoute] string pattern)
    {
        return Task.FromResult(_routeManagementService.GetRouteCalls(new(verb, pattern)));
    }

    [HttpDelete("routes/{verb}/{pattern}/calls")]
    public Task<IActionResult> DeleteRouteCalls([FromRoute] string verb, [FromRoute] string pattern)
    {
        _routeManagementService.DeleteRouteCalls(new(verb, pattern));
        return Task.FromResult<IActionResult>(Ok());
    }

    [HttpPost("routes/{verb}/{pattern}/assert")]
    public IActionResult Assert([FromRoute] string verb, [FromRoute] string pattern)
    {
        throw new NotImplementedException();
    }

    [HttpGet("unmatched")]
    public IActionResult GetUnmatched()
    {
        throw new NotImplementedException();
    }

    [HttpPost("reset")]
    public IActionResult Reset()
    {
        throw new NotImplementedException();
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
