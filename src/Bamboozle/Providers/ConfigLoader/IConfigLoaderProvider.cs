using Bamboozle.Services;

namespace Bamboozle.Providers.ConfigLoader
{
	public interface IConfigLoaderProvider
	{
		string[] ExtensionFilter { get; }
		InitializationModel? LoadFromString(string path);
	}
}
