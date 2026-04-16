
using Bamboozle.Providers;
using Bamboozle.Services;
using System.Text.Json;

var builder = WebApplication.CreateSlimBuilder(args);

builder.Services.AddSingleton<RouteManagementService>();
builder.Services.AddSingleton<ICacheProvider, MemCacheProvider>();
builder.Services.AddMemoryCache();

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

			var match = await routeManagementService.MatchRoute(verb, path);
			if (match is null)
			{
				await context.Response.WriteAsync("null :(");
			}
			else
			{
				match.WithContext(context);
				match.RouteModel.Response.ContentString = JsonSerializer.Serialize(match.QueryParams);
				await context.Response.WriteAsJsonAsync(match);
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