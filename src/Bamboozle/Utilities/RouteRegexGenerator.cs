
using System.Text;
using System.Text.RegularExpressions;
namespace Bamboozle.Utilities
{
	public static partial class RouteRegexGenerator
	{
		private static readonly Dictionary<string, string> ConstraintPatterns = new(StringComparer.OrdinalIgnoreCase)
		{
			["int"] = @"-?\d+",
			["long"] = @"-?\d+",
			["double"] = @"-?\d+(\.\d+)?",
			["decimal"] = @"-?\d+(\.\d+)?",
			["float"] = @"-?\d+(\.\d+)?",
			["bool"] = @"true|false",
			["guid"] = @"[0-9a-fA-F]{8}-[0-9a-fA-F]{4}-[0-9a-fA-F]{4}-[0-9a-fA-F]{4}-[0-9a-fA-F]{12}",
			["alpha"] = @"[a-zA-Z]+",
			["datetime"] = @"\d{4}-\d{2}-\d{2}(T\d{2}:\d{2}:\d{2})?",
		};

		private static string NormalizeUrl(string url) =>
			Regex.Replace(url.Trim('/'), @"/+", "/");

		public static Regex GenerateRouteRegex(string routeTemplate, bool caseInsensitive = true)
		{
			routeTemplate = NormalizeUrl(routeTemplate);

			var tokenSplitter = TemplateRegexPattern();
			var parts = tokenSplitter.Split(routeTemplate);
			var regexBody = new StringBuilder();

			foreach (var part in parts)
			{
				if (part.StartsWith('{') && part.EndsWith('}'))
				{
					var inner = part[1..^1];
					bool optional = inner.EndsWith('?');
					if (optional) inner = inner[..^1];

					var colonIdx = inner.IndexOf(':');
					var paramName = colonIdx >= 0 ? inner[..colonIdx] : inner;
					var constraint = colonIdx >= 0 ? inner[(colonIdx + 1)..] : string.Empty;

					var valuePattern = ConstraintPatterns.TryGetValue(constraint, out var cp)
						? cp
						: @"[^/]+";

					if (optional)
					{
						// Pull the preceding slash into the optional group so that
						// "blog/{slug?}" matches both "blog" and "blog/my-post"
						if (regexBody.Length > 0 && regexBody[^1] == '/')
							regexBody.Length--;

						regexBody.Append($"(?:/(?<{paramName}>{valuePattern}))?");
					}
					else
					{
						regexBody.Append($"(?<{paramName}>{valuePattern})");
					}
				}
				else
				{
					regexBody.Append(Regex.Escape(part));
				}
			}

			var options = RegexOptions.Compiled | RegexOptions.CultureInvariant;
			if (caseInsensitive) options |= RegexOptions.IgnoreCase;

			return new Regex($@"^{regexBody}$", options);
		}

		public static bool TryMatchRoute(
			string routeTemplate,
			string url,
			out Dictionary<string, string> routeValues)
		{
			routeTemplate = NormalizeUrl(routeTemplate);
			url = NormalizeUrl(url);

			if (routeTemplate.Equals(url, StringComparison.OrdinalIgnoreCase))
			{
				routeValues = [];
				return true;
			}

			var regex = GenerateRouteRegex(routeTemplate);
			var match = regex.Match(url);

			if (!match.Success)
			{
				routeValues = [];
				return false;
			}

			routeValues = regex.GetGroupNames()
				.Where(n => !int.TryParse(n, out _))
				.ToDictionary(n => n, n => match.Groups[n].Value);

			return true;
		}

		[GeneratedRegex(@"(\{[^}]+\})")]
		private static partial Regex TemplateRegexPattern();
	}
}
