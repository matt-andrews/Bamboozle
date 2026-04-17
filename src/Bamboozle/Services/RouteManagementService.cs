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
				_routeTrackingProvider.UnmatchedContext(new ContextModel(new RouteDefinition() { Match = key }, new Dictionary<string, string>()));
			}
			else
			{
				_routeTrackingProvider.MatchContext(result);
			}
			return result;
		}

		public IAsyncEnumerable<RouteDefinition> GetAllRoutes()
		{
			return _routeStorageProvider.GetAllRoutes();
		}

		public IEnumerable<ContextModel> GetRouteCalls(MatchKey key)
		{
			return _routeTrackingProvider.GetAllMatchedContexts().Where(w => w.RouteModel.Match == key);
		}

		public void DeleteRouteCalls(MatchKey key)
		{
			_routeTrackingProvider.Delete(key);
		}

		public bool Assert(MatchKey key, Expression<Func<ContextModel, bool>> expression, int expect)
		{
			var calls = GetRouteCalls(key);

			var func = expression.Compile();
			var matches = calls.ToList().Count(func);

			return expect < 0 || matches == expect;
		}

		public IEnumerable<MatchKey> GetUnmatchedRouteCalls()
		{
			return _routeTrackingProvider.GetAllUnmatchedContexts().Select(s => s.RouteModel.Match);
		}

		public async Task Reset()
		{
			await _routeStorageProvider.Reset();
			_routeTrackingProvider.Reset();
		}
	}
}
