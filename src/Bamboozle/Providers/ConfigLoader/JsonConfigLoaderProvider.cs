using Bamboozle.Services;
using System.Text.Json;

namespace Bamboozle.Providers.ConfigLoader
{
	public class JsonConfigLoaderProvider : IConfigLoaderProvider
	{
		public string[] ExtensionFilter { get; } = ["*.json"];

		private readonly JsonSerializerOptions _jsonOptions = new()
		{
			PropertyNamingPolicy = JsonNamingPolicy.CamelCase,
		};

		public InitializationModel? LoadFromString(string path)
		{
			return JsonSerializer.Deserialize<InitializationModel>(path, _jsonOptions);
		}
	}
}
