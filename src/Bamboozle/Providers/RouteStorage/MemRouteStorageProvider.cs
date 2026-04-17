using Bamboozle.Models;
using Bamboozle.Utilities;
using System.Collections.Concurrent;
using System.Diagnostics.CodeAnalysis;

namespace Bamboozle.Providers.RouteStorage
{
    public class MemRouteStorageProvider : IRouteStorageProvider
    {
        private readonly ConcurrentDictionary<string, ConcurrentDictionary<MatchKey, RouteDefinition>> _keyLookup = [];

        public Task<ContextModel?> MatchRoute(MatchKey key)
        {
            if (!TryGetValue(key, out RouteDefinition? result, out Dictionary<string, string>? routeValues))
            {
                return Task.FromResult<ContextModel?>(null);
            }
            return Task.FromResult<ContextModel?>(new ContextModel(result, routeValues));
        }

        public Task SetRoute(RouteDefinition route)
        {
            var keyLookup = GetVerbDict(route.Match);
            if (!keyLookup.TryAdd(route.Match, route))
            {
                throw new InvalidOperationException($"Route already exists: {route.Match}");
            }
            return Task.CompletedTask;
        }

        public async Task DeleteRoute(MatchKey key)
        {
            var keyLookup = GetVerbDict(key);
            if (!keyLookup.Remove(key, out _))
            {
                throw new InvalidOperationException($"Cannot find route to delete: {key}");
            }
        }

        public Task<RouteDefinition?> GetRoute(MatchKey key)
        {
            var keyLookup = GetVerbDict(key);
            if (keyLookup.TryGetValue(key, out RouteDefinition? route))
            {
                return Task.FromResult<RouteDefinition?>(route);
            }
            return Task.FromResult<RouteDefinition?>(null);
        }

        public async IAsyncEnumerable<RouteDefinition> GetAllRoutes()
        {
            foreach (var value in _keyLookup.Values.SelectMany(s => s.Select(ss => ss.Value)))
                yield return value;
        }

        private ConcurrentDictionary<MatchKey, RouteDefinition> GetVerbDict(MatchKey key)
        {
            return _keyLookup.GetOrAdd(key.Verb, static _ => []);
        }

        private bool TryGetValue(
            MatchKey cacheKey,
            [NotNullWhen(true)] out RouteDefinition? model,
            [NotNullWhen(true)] out Dictionary<string, string>? routeValues)
        {
            var keyLookup = GetVerbDict(cacheKey);

            foreach (var (_, value) in keyLookup)
            {
                if (RouteRegexGenerator.TryMatchRoute(value.Match.Pattern, cacheKey.Pattern, out routeValues))
                {
                    model = value;
                    return true;
                }
            }

            model = null;
            routeValues = null;
            return false;
        }
    }
}