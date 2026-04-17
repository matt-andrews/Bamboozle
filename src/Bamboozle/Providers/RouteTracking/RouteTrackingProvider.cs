using Bamboozle.Models;
using System.Collections.Concurrent;

namespace Bamboozle.Providers.RouteTracking;

public class RouteTrackingProvider : IRouteTrackingProvider
{
	private readonly ConcurrentDictionary<Guid, ContextModel> _matchedRoutes = [];
	private readonly ConcurrentDictionary<Guid, ContextModel> _unmatchedRoutes = [];
	public void UnmatchedContext(ContextModel context)
	{
		_unmatchedRoutes.TryAdd(Guid.NewGuid(), context);
	}

	public void MatchContext(ContextModel context)
	{
		_matchedRoutes.TryAdd(Guid.NewGuid(), context);
	}

	public IEnumerable<ContextModel> GetAllMatchedContexts()
	{
		return _matchedRoutes.Values;
	}

	public IEnumerable<ContextModel> GetAllUnmatchedContexts()
	{
		return _unmatchedRoutes.Values;
	}

	public void Delete(MatchKey matchKey)
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
	}

	public void Reset()
	{
		_matchedRoutes.Clear();
		_unmatchedRoutes.Clear();
	}
}
