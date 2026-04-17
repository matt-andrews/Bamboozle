using Bamboozle.Models;
using YamlDotNet.Serialization;
using YamlDotNet.Serialization.NamingConventions;

namespace Bamboozle.Providers.ConfigLoader
{
	public class YamlConfigLoaderProvider : IConfigLoaderProvider
	{
		public string[] ExtensionFilter { get; } = ["*.yml", "*.yaml"];

		private readonly IDeserializer _yamlDeserializer = new DeserializerBuilder()
						.WithNamingConvention(CamelCaseNamingConvention.Instance)
						.Build();

		public ConfigLoaderModel? LoadFromString(string path)
		{
			return _yamlDeserializer.Deserialize<ConfigLoaderModel>(path);
		}
	}
}
