using Bamboozle.Models;
using System.Collections.Concurrent;

namespace Bamboozle.Providers.RouteTracking;

public class RouteTrackingProvider : IRouteTrackingProvider
{
    private readonly ConcurrentDictionary<Guid, ContextModel> _cache = [];
    public void TrackContext(ContextModel context)
    {
        _cache.TryAdd(Guid.NewGuid(), context);
    }
    public IEnumerable<ContextModel> GetAllContexts()
    {
        return _cache.Values;
    }
    public void Delete(MatchKey matchKey)
    {
        foreach (var (key, value) in _cache.ToDictionary())
        {
            if (value.RouteModel.Match == matchKey)
            {
                _cache.TryRemove(key, out _);
            }
        }
    }
}
