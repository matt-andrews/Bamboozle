
var builder = WebApplication.CreateSlimBuilder(args);

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
            // catch-all handler
            await context.Response.WriteAsync("catch-all");
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