namespace Bamboozle.Models
{
	public record ConfigLoaderModel
	{
		public RouteDefinition[] Routes { get; set; } = [];
	}
}
