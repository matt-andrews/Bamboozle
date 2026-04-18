using Bamboozle.Models;
using System.Collections.Concurrent;

namespace Bamboozle.Providers.RouteTracking;

public class RouteTrackingProvider : IRouteTrackingProvider
{
    private readonly ConcurrentDictionary<Guid, ContextModel> _matchedRoutes = [];
    private readonly ConcurrentDictionary<Guid, ContextModel> _unmatchedRoutes = [];
    public Task UnmatchedContext(ContextModel context)
    {
        _unmatchedRoutes.TryAdd(Guid.NewGuid(), context);
        return Task.CompletedTask;
    }

    public Task MatchContext(ContextModel context)
    {
        _matchedRoutes.TryAdd(Guid.NewGuid(), context);
        return Task.CompletedTask;
    }

    public IAsyncEnumerable<ContextModel> GetAllMatchedContexts()
    {
        return _matchedRoutes.Values.ToAsyncEnumerable();
    }

    public IAsyncEnumerable<ContextModel> GetAllUnmatchedContexts()
    {
        return _unmatchedRoutes.Values.ToAsyncEnumerable();
    }

    public Task Delete(MatchKey matchKey)
    {
        foreach (var (key, value) in _matchedRoutes.ToDictionary())
        {
            if (value.RouteModel.Match == matchKey)
            {
                _matchedRoutes.TryRemove(key, out _);
            }
        }

        foreach (var (key, value) in _unmatchedRoutes.ToDictionary())
        {
            if (value.RouteModel.Match == matchKey)
            {
                _matchedRoutes.TryRemove(key, out _);
            }
        }
        return Task.CompletedTask;
    }

    public Task Reset()
    {
        _matchedRoutes.Clear();
        _unmatchedRoutes.Clear();
        return Task.CompletedTask;
    }
}
