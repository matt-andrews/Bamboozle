using Bamboozle.Models;

namespace Bamboozle.Providers
{
	public interface ICacheProvider
	{
		Task SetRoute(RouteModel route);
		Task<ContextModel?> MatchRoute(string verb, string pattern);
	}
}
