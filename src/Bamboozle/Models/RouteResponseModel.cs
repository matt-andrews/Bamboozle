namespace Bamboozle.Models
{
	public class RouteResponseModel
	{
		public int Status { get; set; }
		public Dictionary<string, string> Headers { get; set; }
		public byte[] Content { get; set; }
		public string ContentString { get; set; }
	}
}
