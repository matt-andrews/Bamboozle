using Bamboozle.Models;
using Bamboozle.Utilities;
using Microsoft.Extensions.Caching.Memory;
using System.Collections.Concurrent;
using System.Diagnostics.CodeAnalysis;
using System.Text.RegularExpressions;

namespace Bamboozle.Providers
{
	public class MemCacheProvider(IMemoryCache memcache) : ICacheProvider
	{
		private readonly ConcurrentDictionary<string, ConcurrentDictionary<string, CacheKey>> _keyLookup = [];
		private readonly IMemoryCache _memcache = memcache;

		public Task SetRoute(RouteModel route)
		{
			var key = new CacheKey(route.Match);
			var keyLookup = GetVerbDict(route.Match.Verb);
			keyLookup.TryAdd(key, key);
			_memcache.Set(key, route);
			return Task.CompletedTask;
		}

		public Task<ContextModel?> MatchRoute(string verb, string pattern)
		{
			if (!GetKey(verb, pattern, out CacheKey? key, out Dictionary<string, string>? routeValues))
			{
				return Task.FromResult<ContextModel?>(null);
			}
			if (!_memcache.TryGetValue(key, out RouteModel? result) || result is null)
			{
				return Task.FromResult<ContextModel?>(null);
			}
			return Task.FromResult<ContextModel?>(new ContextModel(result, routeValues));
		}

		private ConcurrentDictionary<string, CacheKey> GetVerbDict(string verb)
		{
			return _keyLookup.GetOrAdd(verb, static _ => []);
		}

		private bool GetKey(
			string verb,
			string pattern,
			[NotNullWhen(true)] out CacheKey? cacheKey,
			[NotNullWhen(true)] out Dictionary<string, string>? routeValues)
		{
			var verbDict = GetVerbDict(verb);

			foreach (var (_, value) in verbDict)
			{
				if (RouteRegexGenerator.TryMatchRoute(value.Match.Pattern, pattern, out routeValues))
				{
					cacheKey = value;
					return true;
				}
			}

			cacheKey = null;
			routeValues = null;
			return false;
		}
	}


	public record CacheKey(MatchModel Match)
	{
		public static implicit operator string(CacheKey key) => $"{key.Match.Verb}/{key.Match.Pattern}";
		public static implicit operator CacheKey(string key)
		{
			string[] parts = key.Split('/');
			return new CacheKey(new MatchModel()
			{
				Verb = parts[0],
				Pattern = string.Join("/", parts.Skip(1))
			});
		}
	}
}