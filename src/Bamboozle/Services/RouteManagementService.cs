using Bamboozle.Models;
using Bamboozle.Providers.RouteStorage;
using Bamboozle.Providers.RouteTracking;
using System.Linq.Dynamic.Core;
using System.Linq.Expressions;

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
            if (result is null)
            {
                await _routeTrackingProvider.UnmatchedContext(new ContextModel(new RouteDefinition() { Match = key }, new Dictionary<string, string>()));
            }
            else
            {
                await _routeTrackingProvider.MatchContext(result);
            }
            return result;
        }

        public IAsyncEnumerable<RouteDefinition> GetAllRoutes()
        {
            return _routeStorageProvider.GetAllRoutes();
        }

        public IAsyncEnumerable<ContextModel> GetRouteCalls(MatchKey key)
        {
            return _routeTrackingProvider.GetAllMatchedContexts().Where(w => w.RouteModel.Match == key);
        }

        public async Task DeleteRouteCalls(MatchKey key)
        {
            await _routeTrackingProvider.Delete(key);
        }

        public async Task<bool> Assert(MatchKey key, Expression<Func<ContextModel, bool>> expression, int expect)
        {
            var calls = GetRouteCalls(key);

            var func = expression.Compile();
            var matches = await calls.CountAsync(func);

            return expect < 0 || matches == expect;
        }

        public IAsyncEnumerable<MatchKey> GetUnmatchedRouteCalls()
        {
            return _routeTrackingProvider.GetAllUnmatchedContexts().Select(s => s.RouteModel.Match);
        }

        public async Task Reset()
        {
            await _routeStorageProvider.Reset();
            await _routeTrackingProvider.Reset();
        }
    }
}
