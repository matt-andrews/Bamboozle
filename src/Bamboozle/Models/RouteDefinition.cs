namespace Bamboozle.Models
{
	public record RouteDefinition
	{
		public MatchKey Match { get; set; }
		public ResponseDefinition Response { get; set; }
	}
}
