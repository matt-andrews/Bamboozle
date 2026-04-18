using Bamboozle.Models;

namespace Bamboozle.Providers.RouteTracking;

public interface IRouteTrackingProvider
{
    Task MatchContext(ContextModel context);
    Task UnmatchedContext(ContextModel context);
    IAsyncEnumerable<ContextModel> GetAllMatchedContexts();
    IAsyncEnumerable<ContextModel> GetAllUnmatchedContexts();
    Task Delete(MatchKey matchKey);
    Task Reset();
}
