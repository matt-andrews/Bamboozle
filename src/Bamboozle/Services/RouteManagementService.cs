using Bamboozle.Models;
using Bamboozle.Providers.RouteStorage;
using Bamboozle.Providers.RouteTracking;

namespace Bamboozle.Services
{
    public class RouteManagementService(IRouteStorageProvider routeStorageProvider, IRouteTrackingProvider routeTrackingProvider)
    {
        private readonly IRouteStorageProvider _routeStorageProvider = routeStorageProvider;
        private readonly IRouteTrackingProvider _routeTrackingProvider = routeTrackingProvider;

        public async Task DeleteRoute(MatchKey key)
        {
            await _routeStorageProvider.DeleteRoute(key);
        }

        public async Task<RouteDefinition?> GetRoute(MatchKey key)
        {
            return await _routeStorageProvider.GetRoute(key);
        }

        public async Task SetRoute(RouteDefinition route)
        {
            await _routeStorageProvider.SetRoute(route);
        }

        public async Task<ContextModel?> MatchRoute(MatchKey key)
        {
            var result = await _routeStorageProvider.MatchRoute(key);
            _routeTrackingProvider.TrackContext(result ?? new ContextModel(new RouteDefinition() { Match = key }, new Dictionary<string, string>()));
            return result;
        }

        public IAsyncEnumerable<RouteDefinition> GetAllRoutes()
        {
            return _routeStorageProvider.GetAllRoutes();
        }

        public IEnumerable<ContextModel> GetRouteCalls(MatchKey key)
        {
            return _routeTrackingProvider.GetAllContexts().Where(w => w.RouteModel.Match == key);
        }

        public void DeleteRouteCalls(MatchKey key)
        {
            _routeTrackingProvider.Delete(key);
        }
    }
}
