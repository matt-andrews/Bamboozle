using System.Net.Http;
using Bamboozle.Core;

namespace Bamboozle.Tests;

public class BamboozleFixture : IDisposable
{
    public BamboozleHttpClient Bamboozle { get; }
    public HttpClient MockClient { get; }

    public BamboozleFixture()
    {
        var controlClient = new HttpClient { BaseAddress = new Uri("http://localhost:19090") };
        Bamboozle = new BamboozleHttpClient(controlClient);
        
        MockClient = new HttpClient { BaseAddress = new Uri("http://localhost:18080") };
    }

    public void Dispose()
    {
        MockClient.Dispose();
        // Since we don't own the underlying HttpClient of BamboozleHttpClient natively unless we dispose the controlClient we created.
        // For testing, this is fine to not fully dispose the inner client if it's not exposed, or we can just ignore it for the fixture lifecycle.
    }
}
