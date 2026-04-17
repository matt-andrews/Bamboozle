using Bamboozle.Models;
using Bamboozle.Providers.RouteStorage;
using Microsoft.Extensions.Caching.Memory;
using System;
using System.Collections.Generic;
using System.Text;

namespace Bamboozle.Tests
{
	public class MemCacheProviderTests
	{
		private readonly MemRouteStorageProvider _sut;

		public MemCacheProviderTests()
		{
			_sut = new MemRouteStorageProvider();
		}

		// -------------------------------------------------------------------------
		// Helpers
		// -------------------------------------------------------------------------

		private static RouteDefinition MakeRoute(string verb, string pattern) =>
			new() { Verb = verb, Pattern = pattern };

		// -------------------------------------------------------------------------
		// SetRoute
		// -------------------------------------------------------------------------

		public class SetRoute : MemCacheProviderTests
		{
			[Fact]
			public async Task Returns_completed_task()
			{
				var task = _sut.SetRoute(MakeRoute("GET", "/users"));
				await task;
				Assert.True(task.IsCompletedSuccessfully);
			}

			[Fact]
			public async Task Stored_route_is_subsequently_matchable()
			{
				await _sut.SetRoute(MakeRoute("GET", "/users"));
				var result = await _sut.MatchRoute("GET", "/users");
				Assert.NotNull(result);
			}

			[Fact]
			public async Task Calling_set_twice_with_same_key_does_throw()
			{
				var route = MakeRoute("GET", "/users");
				await _sut.SetRoute(route);
				var ex = await Record.ExceptionAsync(() => _sut.SetRoute(route));
				Assert.NotNull(ex);
			}

			[Fact]
			public async Task Routes_under_different_verbs_are_stored_independently()
			{
				await _sut.SetRoute(MakeRoute("GET", "/users"));
				await _sut.SetRoute(MakeRoute("POST", "/users"));

				var getResult = await _sut.MatchRoute("GET", "/users");
				var postResult = await _sut.MatchRoute("POST", "/users");

				Assert.NotNull(getResult);
				Assert.NotNull(postResult);
			}
		}

		// -------------------------------------------------------------------------
		// MatchRoute — miss cases
		// -------------------------------------------------------------------------

		public class MatchRoute_Misses : MemCacheProviderTests
		{
			[Fact]
			public async Task Returns_null_when_cache_is_empty()
			{
				var result = await _sut.MatchRoute("GET", "/users");
				Assert.Null(result);
			}

			[Fact]
			public async Task Returns_null_for_unknown_verb()
			{
				await _sut.SetRoute(MakeRoute("GET", "/users"));
				var result = await _sut.MatchRoute("DELETE", "/users");
				Assert.Null(result);
			}

			[Fact]
			public async Task Returns_null_for_wrong_verb_on_known_pattern()
			{
				await _sut.SetRoute(MakeRoute("POST", "/users"));
				var result = await _sut.MatchRoute("GET", "/users");
				Assert.Null(result);
			}

			[Fact]
			public async Task Returns_null_when_pattern_does_not_match()
			{
				await _sut.SetRoute(MakeRoute("GET", "/users"));
				var result = await _sut.MatchRoute("GET", "/orders");
				Assert.Null(result);
			}

			[Fact]
			public async Task Returns_null_for_partial_pattern_match()
			{
				await _sut.SetRoute(MakeRoute("GET", "/users/{id}"));
				// Missing the required segment entirely
				var result = await _sut.MatchRoute("GET", "/users");
				Assert.Null(result);
			}
		}

		// -------------------------------------------------------------------------
		// MatchRoute — hit cases (static patterns)
		// -------------------------------------------------------------------------

		public class MatchRoute_StaticPatterns : MemCacheProviderTests
		{
			[Fact]
			public async Task Matches_exact_static_route()
			{
				var route = MakeRoute("GET", "/users");
				await _sut.SetRoute(route);

				var result = await _sut.MatchRoute("GET", "/users");

				Assert.NotNull(result);
				Assert.Equal(route, result.RouteModel);
			}

			[Fact]
			public async Task Match_returns_empty_route_values_for_static_pattern()
			{
				await _sut.SetRoute(MakeRoute("GET", "/health"));
				var result = await _sut.MatchRoute("GET", "/health");

				Assert.NotNull(result?.RouteValues);
				Assert.Empty(result.RouteValues);
			}

			[Fact]
			public async Task Matches_correct_route_among_multiple_static_routes()
			{
				var usersRoute = MakeRoute("GET", "/users");
				var ordersRoute = MakeRoute("GET", "/orders");
				await _sut.SetRoute(usersRoute);
				await _sut.SetRoute(ordersRoute);

				var result = await _sut.MatchRoute("GET", "/orders");

				Assert.NotNull(result);
				Assert.Equal(ordersRoute, result.RouteModel);
			}

			[Fact]
			public async Task Verb_matching_is_case_sensitive()
			{
				await _sut.SetRoute(MakeRoute("GET", "/users"));

				var result = await _sut.MatchRoute("get", "/users");

				// Verbs are stored as provided; "get" != "GET"
				Assert.Null(result);
			}
		}

		// -------------------------------------------------------------------------
		// MatchRoute — parameterized patterns
		// -------------------------------------------------------------------------

		public class MatchRoute_ParameterizedPatterns : MemCacheProviderTests
		{
			[Fact]
			public async Task Extracts_single_route_parameter()
			{
				await _sut.SetRoute(MakeRoute("GET", "/users/{id}"));

				var result = await _sut.MatchRoute("GET", "/users/42");

				Assert.NotNull(result?.RouteValues);
				Assert.Equal("42", result.RouteValues["id"]);
			}

			[Fact]
			public async Task Extracts_multiple_route_parameters()
			{
				await _sut.SetRoute(MakeRoute("GET", "/users/{userId}/orders/{orderId}"));

				var result = await _sut.MatchRoute("GET", "/users/7/orders/99");

				Assert.NotNull(result?.RouteValues);
				Assert.Equal("7", result.RouteValues["userId"]);
				Assert.Equal("99", result.RouteValues["orderId"]);
			}

			[Fact]
			public async Task Returns_correct_route_model_alongside_route_values()
			{
				var route = MakeRoute("DELETE", "/items/{id}");
				await _sut.SetRoute(route);

				var result = await _sut.MatchRoute("DELETE", "/items/123");

				Assert.NotNull(result?.RouteValues);
				Assert.Equal(route, result.RouteModel);
				Assert.Equal("123", result.RouteValues["id"]);
			}

			[Fact]
			public async Task Does_not_match_parameterized_route_across_wrong_verb()
			{
				await _sut.SetRoute(MakeRoute("PUT", "/users/{id}"));

				var result = await _sut.MatchRoute("PATCH", "/users/42");

				Assert.Null(result);
			}

			[Fact]
			public async Task Selects_correct_parameterized_route_among_several()
			{
				var getUserRoute = MakeRoute("GET", "/users/{id}");
				var getOrderRoute = MakeRoute("GET", "/orders/{id}");
				var postUserRoute = MakeRoute("POST", "/users/{id}");
				await _sut.SetRoute(getUserRoute);
				await _sut.SetRoute(getOrderRoute);
				await _sut.SetRoute(postUserRoute);

				var result = await _sut.MatchRoute("GET", "/orders/55");

				Assert.NotNull(result?.RouteValues);
				Assert.Equal(getOrderRoute, result.RouteModel);
				Assert.Equal("55", result.RouteValues["id"]);
			}
		}

		// -------------------------------------------------------------------------
		// Concurrency
		// -------------------------------------------------------------------------

		public class Concurrency : MemCacheProviderTests
		{
			[Fact]
			public async Task Concurrent_sets_do_not_throw()
			{
				var tasks = Enumerable.Range(0, 50)
					.Select(i => _sut.SetRoute(MakeRoute("GET", $"/route/{i}")));

				var ex = await Record.ExceptionAsync(() => Task.WhenAll(tasks));
				Assert.Null(ex);
			}

			[Fact]
			public async Task Concurrent_sets_and_matches_do_not_throw()
			{
				// Pre-populate so matches have something to find
				await Task.WhenAll(Enumerable.Range(0, 20)
					.Select(i => _sut.SetRoute(MakeRoute("GET", $"/route/{i}"))));

				var writes = Enumerable.Range(20, 20)
					.Select(i => _sut.SetRoute(MakeRoute("GET", $"/route/{i}")));

				var reads = Enumerable.Range(0, 20)
					.Select(i => _sut.MatchRoute("GET", $"/route/{i}"));

				var ex = await Record.ExceptionAsync(() =>
					Task.WhenAll(writes.Cast<Task>().Concat(reads.Cast<Task>())));

				Assert.Null(ex);
			}
		}
	}
}