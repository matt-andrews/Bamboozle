using Bamboozle.Models;

namespace Bamboozle.Providers.ConfigLoader
{
	public interface IConfigLoaderProvider
	{
		string[] ExtensionFilter { get; }
		ConfigLoaderModel? LoadFromString(string path);
	}
}
