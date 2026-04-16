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
	public IActionResult PostRoutes()
	{
		throw new NotImplementedException();
	}

	[HttpPut("routes/{id}")]
	public IActionResult PutRoutes(string id)
	{
		throw new NotImplementedException();
	}

	[HttpDelete("routes")]
	public IActionResult DeleteRoutes()
	{
		throw new NotImplementedException();
	}

	[HttpGet("routes")]
	public async Task<IActionResult> GetRoutes()
	{
		await _routeManagementService.SetRoute(new Models.RouteModel()
		{
			Id = "test",
			Match = new() { Verb = "GET", Pattern = "test/{name}" },
			Response = new()
			{
				Status = 200,
				Headers = [],
				Content = []
			}
		});
		return Ok();
	}

	[HttpGet("routes/{id}/calls")]
	public IActionResult GetRouteCalls(string id)
	{
		throw new NotImplementedException();
	}

	[HttpDelete("routes/{id}/calls")]
	public IActionResult DeleteRouteCalls(string id)
	{
		throw new NotImplementedException();
	}

	[HttpPost("routes/{id}/assert")]
	public IActionResult Assert(string id)
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
	public IActionResult Health()
	{
		throw new NotImplementedException();
	}

	[HttpGet("version")]
	public Task<IActionResult> Version()
	{
		return Task.FromResult<IActionResult>(Ok(typeof(ControlController).Assembly.GetName().Version?.ToString() ?? "0.0.0"));
	}
}
