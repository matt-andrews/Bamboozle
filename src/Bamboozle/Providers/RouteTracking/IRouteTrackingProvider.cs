using Bamboozle.Models;

namespace Bamboozle.Providers.RouteTracking;

public interface IRouteTrackingProvider
{
    void TrackContext(ContextModel context);
    IEnumerable<ContextModel> GetAllContexts();
    void Delete(MatchKey matchKey);
}
