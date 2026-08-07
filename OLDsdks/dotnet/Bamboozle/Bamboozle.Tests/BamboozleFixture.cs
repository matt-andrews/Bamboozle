using System.Net.Http;
using Bamboozle.Core;

namespace Bamboozle.Tests;

public class BamboozleFixture : IDisposable
{
    private readonly HttpClient _controlClient;
    public BamboozleHttpClient Bamboozle { get; }
    public HttpClient MockClient { get; }
    public BamboozleFixture()
    {
        _controlClient = new HttpClient { BaseAddress = new Uri("http://localhost:19090") };
        Bamboozle = new BamboozleHttpClient(_controlClient);
        MockClient = new HttpClient { BaseAddress = new Uri("http://localhost:18080") };
    }
    public void Dispose()
    {
        MockClient.Dispose();
        _controlClient.Dispose();
    }
}
