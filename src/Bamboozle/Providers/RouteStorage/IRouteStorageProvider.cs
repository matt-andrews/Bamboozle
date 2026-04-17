using Bamboozle.Models;

namespace Bamboozle.Providers.RouteStorage
{
	public interface IRouteStorageProvider
	{
		Task Reset();

		Task DeleteRoute(MatchKey key);
        Task SetRoute(RouteDefinition route);
		Task<ContextModel?> MatchRoute(MatchKey key);
		Task<RouteDefinition?> GetRoute(MatchKey key);
        IAsyncEnumerable<RouteDefinition> GetAllRoutes();
    }
}
