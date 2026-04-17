using Bamboozle.Models;
using Bamboozle.Providers.ConfigLoader;
using Bamboozle.Providers.RouteStorage;
using Bamboozle.Providers.RouteTracking;
using Bamboozle.Services;
using System.Text.Json;

var builder = WebApplication.CreateSlimBuilder(args);

builder.Services.AddSingleton<RouteManagementService>();
builder.Services.AddSingleton<IRouteStorageProvider, MemRouteStorageProvider>();
builder.Services.AddSingleton<IRouteTrackingProvider, RouteTrackingProvider>();

builder.Services.AddSingleton<ConfigLoaderService>()
	.AddConfigLoaderProvider<JsonConfigLoaderProvider>()
	.AddConfigLoaderProvider<YamlConfigLoaderProvider>()
	.AddHostedService(provider => provider.GetRequiredService<ConfigLoaderService>());

builder.Services.AddControllers();

builder.WebHost.ConfigureKestrel(options =>
{
	options.ListenAnyIP(8080);
	options.ListenAnyIP(9090);
});

builder.Services.AddEndpointsApiExplorer();
builder.Services.AddSwaggerGen();

var app = builder.Build();

if (app.Environment.IsDevelopment())
{
	app.UseSwagger();
	app.UseSwaggerUI();
}

app.UseWhen(
	ctx => ctx.Connection.LocalPort == 8080,
	branch =>
	{
		branch.Run(async context =>
		{
			var provider = context.RequestServices;
			var routeManagementService = provider.GetRequiredService<RouteManagementService>();

			string path = context.Request.Path.ToString();
			string verb = context.Request.Method;
			MatchKey key = new(verb, path);

			var match = await routeManagementService.MatchRoute(key);
			if (match is null)
			{
				context.Response.StatusCode = 404;
				return;
			}
			else
			{
				match.WithContext(context);
				var responseObj = match.RouteModel.Response.WithTemplate(match);
				_ = int.TryParse(responseObj.Status, out int statusCode);
				context.Response.StatusCode = statusCode;

				switch (responseObj.Headers["Content-Type"])
				{
					case "application/json":
						await context.Response.WriteAsJsonAsync(
								JsonSerializer.Deserialize<object>(responseObj.Content ?? "{}")
							);
						break;
					default:
						await context.Response.WriteAsync(responseObj.Content ?? "");
						break;
				}

				return;
			}
		});
	}
);

app.UseWhen(
	ctx => ctx.Connection.LocalPort == 9090,
	branch =>
	{
		branch.UseRouting();
		branch.UseEndpoints(e => e.MapControllers());
	}
);

await app.RunAsync();