using Bamboozle.Models;
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

		public ConfigLoaderModel? LoadFromString(string path)
		{
			return JsonSerializer.Deserialize<ConfigLoaderModel>(path, _jsonOptions);
		}
	}
}
