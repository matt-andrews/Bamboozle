namespace Bamboozle.Models
{
	public sealed class ContextModel
	{
		public IReadOnlyDictionary<string, string>? QueryParams { get; private set; }
		public IReadOnlyDictionary<string, string>? Headers { get; private set; }
		public IReadOnlyDictionary<string, string>? RouteValues { get; }
		public RouteDefinition RouteModel { get; }
		public ContextModel(RouteDefinition route, IReadOnlyDictionary<string, string> routeValues)
		{
			RouteModel = route;
			RouteValues = routeValues;
		}
		public void WithContext(HttpContext context)
		{
			QueryParams = context.Request.Query.ToDictionary(k => k.Key, v => v.Value.ToString());
			Headers = context.Request.Headers.ToDictionary(k => k.Key, v => v.Value.ToString());
		}
	}
}
