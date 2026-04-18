using Bamboozle.Models;
using Bamboozle.Providers.ConfigLoader;

namespace Bamboozle.Services
{
    public class ConfigLoaderService(
        IConfiguration config,
        ILogger<ConfigLoaderService> logger,
        RouteManagementService routeManagementService,
        IEnumerable<IConfigLoaderProvider> configProviders
        ) : IHostedService
    {
        private readonly IConfiguration _config = config;
        private readonly ILogger<ConfigLoaderService> _logger = logger;
        private readonly RouteManagementService _routeManagementService = routeManagementService;
        private readonly IEnumerable<IConfigLoaderProvider> _configProviders = configProviders;
        private readonly bool _throwOnError
            = config[Consts.Config.RouteConfigThrowOnError]?
                .Equals("true", StringComparison.OrdinalIgnoreCase)
                    ?? false;
        public async Task Init()
        {
            string[] folders = _config.GetSection(Consts.Config.RouteConfigFolders).Get<string[]>() ?? [];

            foreach (var folder in folders)
            {
                if (!Directory.Exists(folder))
                {
                    _logger.LogWarning("Cannot find given directory on the file system: {folder}", folder);
                    if (_throwOnError)
                        throw new InvalidOperationException($"Cannot find given directory on the file system: {folder}");
                    continue;
                }

                foreach (var provider in _configProviders)
                {
                    await LoadFiles(folder, provider);
                }
            }
        }

        private async Task LoadFiles(string folder, IConfigLoaderProvider provider)
        {
            foreach (var ext in provider.ExtensionFilter)
            {
                var files = Directory.GetFiles(folder, ext);
                foreach (var file in files)
                {
                    try
                    {
                        ConfigLoaderModel? obj = provider.LoadFromString(await File.ReadAllTextAsync(file));

                        if (obj is null) continue;

                        foreach (var route in obj.Routes)
                        {
                            await _routeManagementService.SetRoute(route);
                            if (_logger.IsEnabled(LogLevel.Information))
                                _logger.LogInformation("Created route: {route}", route.Match);
                        }
                    }
                    catch (Exception ex)
                    {
                        _logger.LogError(ex, "Failed to load file {file} with message {msg}", file, ex.Message);
                        if (_throwOnError) throw;
                    }
                }
            }
        }

        public async Task StartAsync(CancellationToken cancellationToken)
        {
            _logger.LogInformation("Starting initialization");
            await Init();
            _logger.LogInformation("Finished intialization");
        }

        public Task StopAsync(CancellationToken cancellationToken)
        {
            return Task.CompletedTask;
        }
    }

    public static class ConfigLoaderServiceExt
    {
        public static IServiceCollection AddConfigLoaderProvider<TConcrete>(this IServiceCollection services)
            where TConcrete : class, IConfigLoaderProvider
        {
            services.AddSingleton<IConfigLoaderProvider, TConcrete>();
            return services;
        }
    }
}
