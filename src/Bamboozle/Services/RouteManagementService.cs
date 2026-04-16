using Bamboozle.Models;
using Bamboozle.Providers;

namespace Bamboozle.Services
{
	public class RouteManagementService(ICacheProvider cacheProvider)
	{
		private readonly ICacheProvider _cacheProvider = cacheProvider;

		public async Task SetRoute(RouteModel route)
		{
			await _cacheProvider.SetRoute(route);
		}

		public async Task<ContextModel?> MatchRoute(string verb, string pattern)
		{
			return await _cacheProvider.MatchRoute(verb, pattern);
		}
	}
}
