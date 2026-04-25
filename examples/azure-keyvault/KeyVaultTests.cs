using Azure.Core;
using Azure.Security.KeyVault.Secrets;

namespace AzureKeyVaultExample;

public class MockTokenCredential : TokenCredential
{
    public override AccessToken GetToken(TokenRequestContext requestContext, CancellationToken cancellationToken)
    {
        return new AccessToken("mock-token", DateTimeOffset.UtcNow.AddHours(1));
    }

    public override ValueTask<AccessToken> GetTokenAsync(TokenRequestContext requestContext, CancellationToken cancellationToken)
    {
        return new ValueTask<AccessToken>(new AccessToken("mock-token", DateTimeOffset.UtcNow.AddHours(1)));
    }
}

public class KeyVaultTests
{
    private readonly SecretClient _client;

    public KeyVaultTests()
    {
        var options = new SecretClientOptions
        {
            Retry = { MaxRetries = 0 },
            // Required when pointing at a non-Azure endpoint.
            DisableChallengeResourceVerification = true
        };

        _client = new SecretClient(
            new Uri("https://localhost:44044"),
            new MockTokenCredential(),
            options
        );
    }

    [Fact]
    public async Task GetSecret_ShouldReturnMockedValue()
    {
        KeyVaultSecret secret = await _client.GetSecretAsync("my-database-password");

        Assert.NotNull(secret);
        Assert.Equal("my-database-password", secret.Name);
        Assert.Equal("super-secret-password123", secret.Value);
    }

    [Fact]
    public async Task SetSecret_ShouldReturnNewlyCreatedSecret()
    {
        KeyVaultSecret secret = await _client.SetSecretAsync("new-api-key", "some-new-value");

        Assert.NotNull(secret);
        Assert.Equal("new-api-key", secret.Name);
        Assert.Equal("some-new-value", secret.Value);
    }
}
