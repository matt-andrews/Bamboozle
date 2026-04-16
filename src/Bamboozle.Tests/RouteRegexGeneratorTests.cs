using Bamboozle.Utilities;

namespace Bamboozle.Tests
{

	public class RouteRegexGeneratorTests
	{
		// -------------------------------------------------------
		// GenerateRouteRegex — regex shape
		// -------------------------------------------------------

		[Fact]
		public void GenerateRouteRegex_StaticRoute_MatchesExactString()
		{
			var regex = RouteRegexGenerator.GenerateRouteRegex("version");
			Assert.Matches(regex, "version");
		}

		[Fact]
		public void GenerateRouteRegex_StaticRoute_DoesNotMatchDifferentString()
		{
			var regex = RouteRegexGenerator.GenerateRouteRegex("version");
			Assert.DoesNotMatch(regex, "v2");
		}

		[Fact]
		public void GenerateRouteRegex_SingleParam_MatchesAnySegment()
		{
			var regex = RouteRegexGenerator.GenerateRouteRegex("api/{id}/name");
			Assert.Matches(regex, "api/42/name");
			Assert.Matches(regex, "api/abc-xyz/name");
			Assert.Matches(regex, "api/some_value/name");
		}

		[Fact]
		public void GenerateRouteRegex_SingleParam_DoesNotMatchMissingSegment()
		{
			var regex = RouteRegexGenerator.GenerateRouteRegex("api/{id}/name");
			Assert.DoesNotMatch(regex, "api//name");
			Assert.DoesNotMatch(regex, "api/name");
		}

		[Fact]
		public void GenerateRouteRegex_MultipleParams_MatchesAllSegments()
		{
			var regex = RouteRegexGenerator.GenerateRouteRegex("controller/{action}/{id}");
			Assert.Matches(regex, "controller/edit/42");
			Assert.Matches(regex, "controller/delete/abc");
		}

		[Fact]
		public void GenerateRouteRegex_MultipleParams_DoesNotMatchExtraSegments()
		{
			var regex = RouteRegexGenerator.GenerateRouteRegex("controller/{action}/{id}");
			Assert.DoesNotMatch(regex, "controller/edit/42/extra");
		}

		[Fact]
		public void GenerateRouteRegex_IsCaseInsensitiveByDefault()
		{
			var regex = RouteRegexGenerator.GenerateRouteRegex("api/{id}/name");
			Assert.Matches(regex, "API/42/NAME");
			Assert.Matches(regex, "Api/42/Name");
		}

		[Fact]
		public void GenerateRouteRegex_CaseSensitiveWhenRequested()
		{
			var regex = RouteRegexGenerator.GenerateRouteRegex("api/{id}/name", caseInsensitive: false);
			Assert.Matches(regex, "api/42/name");
			Assert.DoesNotMatch(regex, "API/42/NAME");
		}

		[Fact]
		public void GenerateRouteRegex_ParamDoesNotMatchAcrossSlashes()
		{
			// {id} should not consume "12/extra" — it stops at the slash
			var regex = RouteRegexGenerator.GenerateRouteRegex("api/{id}/name");
			Assert.DoesNotMatch(regex, "api/12/extra/name");
		}

		// -------------------------------------------------------
		// Constraint: :int / :long
		// -------------------------------------------------------

		[Theory]
		[InlineData("orders/{id:int}/detail", "orders/99/detail", true)]
		[InlineData("orders/{id:int}/detail", "orders/-5/detail", true)]
		[InlineData("orders/{id:int}/detail", "orders/abc/detail", false)]
		[InlineData("orders/{id:int}/detail", "orders/1.5/detail", false)]
		public void GenerateRouteRegex_IntConstraint(string template, string url, bool shouldMatch)
		{
			var regex = RouteRegexGenerator.GenerateRouteRegex(template);
			Assert.Equal(shouldMatch, regex.IsMatch(url));
		}

		[Theory]
		[InlineData("items/{id:guid}", "items/3f2504e0-4f89-11d3-9a0c-0305e82c3301", true)]
		[InlineData("items/{id:guid}", "items/not-a-guid", false)]
		public void GenerateRouteRegex_GuidConstraint(string template, string url, bool shouldMatch)
		{
			var regex = RouteRegexGenerator.GenerateRouteRegex(template);
			Assert.Equal(shouldMatch, regex.IsMatch(url));
		}

		[Theory]
		[InlineData("users/{name:alpha}", "users/John", true)]
		[InlineData("users/{name:alpha}", "users/john", true)]
		[InlineData("users/{name:alpha}", "users/123", false)]
		[InlineData("users/{name:alpha}", "users/jo-hn", false)]
		public void GenerateRouteRegex_AlphaConstraint(string template, string url, bool shouldMatch)
		{
			var regex = RouteRegexGenerator.GenerateRouteRegex(template);
			Assert.Equal(shouldMatch, regex.IsMatch(url));
		}

		[Theory]
		[InlineData("flag/{val:bool}", "flag/true", true)]
		[InlineData("flag/{val:bool}", "flag/false", true)]
		[InlineData("flag/{val:bool}", "flag/yes", false)]
		[InlineData("flag/{val:bool}", "flag/1", false)]
		public void GenerateRouteRegex_BoolConstraint(string template, string url, bool shouldMatch)
		{
			var regex = RouteRegexGenerator.GenerateRouteRegex(template);
			Assert.Equal(shouldMatch, regex.IsMatch(url));
		}

		// -------------------------------------------------------
		// Optional parameters
		// -------------------------------------------------------

		[Fact]
		public void GenerateRouteRegex_OptionalParam_MatchesWithValue()
		{
			var regex = RouteRegexGenerator.GenerateRouteRegex("blog/{slug?}");
			Assert.Matches(regex, "blog/my-post");
		}


		// -------------------------------------------------------
		// TryMatchRoute — value extraction
		// -------------------------------------------------------

		[Fact]
		public void TryMatchRoute_SingleParam_ExtractsValue()
		{
			var matched = RouteRegexGenerator.TryMatchRoute("api/{id}/name", "api/42/name", out var values);
			Assert.True(matched);
			Assert.Equal("42", values["id"]);
		}

		[Fact]
		public void TryMatchRoute_MultipleParams_ExtractsAllValues()
		{
			var matched = RouteRegexGenerator.TryMatchRoute(
				"controller/{action}/{id}", "controller/edit/99", out var values);
			Assert.True(matched);
			Assert.Equal("edit", values["action"]);
			Assert.Equal("99", values["id"]);
		}

		[Fact]
		public void TryMatchRoute_NoMatch_ReturnsFalseAndEmptyDict()
		{
			var matched = RouteRegexGenerator.TryMatchRoute("api/{id}/name", "api/name", out var values);
			Assert.False(matched);
			Assert.Empty(values);
		}

		[Fact]
		public void TryMatchRoute_OptionalParam_ExtractsValueWhenPresent()
		{
			var matched = RouteRegexGenerator.TryMatchRoute("blog/{slug?}", "blog/hello-world", out var values);
			Assert.True(matched);
			Assert.Equal("hello-world", values["slug"]);
		}

		[Fact]
		public void TryMatchRoute_OptionalParam_EmptyStringWhenAbsent()
		{
			// all three forms are equivalent after normalization
			foreach (var url in new[] { "blog", "blog/", "/blog/" })
			{
				var matched = RouteRegexGenerator.TryMatchRoute("blog/{slug?}", url, out var values);
				Assert.True(matched);
				Assert.Equal(string.Empty, values["slug"]);
			}
		}

		[Fact]
		public void TryMatchRoute_StaticRoute_NoRouteValues()
		{
			var matched = RouteRegexGenerator.TryMatchRoute("version", "version", out var values);
			Assert.True(matched);
			Assert.Empty(values);
		}

		[Fact]
		public void TryMatchRoute_LeadingSlash_IsStripped()
		{
			// URLs passed with a leading slash should still match
			var matched = RouteRegexGenerator.TryMatchRoute("api/{id}/name", "/api/5/name", out var values);
			Assert.True(matched);
			Assert.Equal("5", values["id"]);
		}

		// -------------------------------------------------------
		// Special characters in static segments
		// -------------------------------------------------------

		[Fact]
		public void GenerateRouteRegex_DotInTemplate_TreatedAsLiteral()
		{
			// "v1.0" — the dot must NOT match any character
			var regex = RouteRegexGenerator.GenerateRouteRegex("v1.0/{id}");
			Assert.Matches(regex, "v1.0/42");
			Assert.DoesNotMatch(regex, "v1X0/42");
		}

		[Fact]
		public void GenerateRouteRegex_HyphenInTemplate_TreatedAsLiteral()
		{
			var regex = RouteRegexGenerator.GenerateRouteRegex("api-v2/{id}");
			Assert.Matches(regex, "api-v2/test");
			Assert.DoesNotMatch(regex, "apiv2/test");
		}
	}
}
