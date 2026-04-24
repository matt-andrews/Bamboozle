using System;
using System.Threading;
using System.Threading.Tasks;
using Azure.Core;
using Azure.Security.KeyVault.Secrets;
using Xunit;

namespace AzureKeyVaultExample;

// 1. MockTokenCredential bypasses the Entra ID / OAuth authentication process
// so we don't have to mock login.microsoftonline.com.
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
        // Disable retry logic for fast testing
        var options = new SecretClientOptions
        {
            Retry = { MaxRetries = 0 }
        };

        // 2. Point the SecretClient to the Bamboozle HTTPS endpoint
        _client = new SecretClient(
            new Uri("https://localhost:8080"), 
            new MockTokenCredential(),
            options
        );
    }

    [Fact]
    public async Task GetSecret_ShouldReturnMockedValue()
    {
        // Act
        KeyVaultSecret secret = await _client.GetSecretAsync("my-database-password");

        // Assert
        Assert.NotNull(secret);
        Assert.Equal("my-database-password", secret.Name);
        Assert.Equal("super-secret-password123", secret.Value);
    }

    [Fact]
    public async Task SetSecret_ShouldReturnNewlyCreatedSecret()
    {
        // Act
        KeyVaultSecret secret = await _client.SetSecretAsync("new-api-key", "some-new-value");

        // Assert
        Assert.NotNull(secret);
        Assert.Equal("new-api-key", secret.Name);
        Assert.Equal("some-new-value", secret.Value);
    }
}
