using Bamboozle.Models;

namespace Bamboozle.Providers.RouteTracking;

public interface IRouteTrackingProvider
{
	void MatchContext(ContextModel context);
	void UnmatchedContext(ContextModel context);
	IEnumerable<ContextModel> GetAllMatchedContexts();
	IEnumerable<ContextModel> GetAllUnmatchedContexts();
	void Delete(MatchKey matchKey);
	void Reset();
}
