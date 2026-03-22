#:sdk Aspire.AppHost.Sdk@13.1.3
#:package Aspire.Hosting.PostgreSQL@*
#:package Aspire.Hosting.JavaScript@13.1.3
#:package Aspire.Hosting.Docker@13.1.3-preview.1.26166.8 

var builder = DistributedApplication.CreateBuilder(args);
var compose = builder.AddDockerComposeEnvironment("compose");

var postgresUser = builder.AddParameter("postgres-user", "loco");
var postgresPassword = builder.AddParameter("postgres-password", "loco", secret: true);
var webApiEnableDrift = builder.AddParameter("enable-drift", "false");
var heliusApiKey = builder.AddParameter("helius-api-key", "COPY_CREDS_HERE", secret: true);
var walletPrivateKey = builder.AddParameter("wallet-private-key", "COPY_CREDS_HERE", secret: true);
var aiGatewayUrl = builder.AddParameter("ai-gateway-url", "COPY_CREDS_HERE");
var aiGatewayKey = builder.AddParameter("ai-gateway-key", "COPY_CREDS_HERE", secret: true);
var aiGatewayModel = builder.AddParameter("ai-gateway-model", "anthropic/claude-sonnet-4-6");
var nextPublicTradingApiUrl = builder.AddParameter("next-public-trading-api-url", "http://localhost:5150");

var postgres = builder.AddPostgres("postgres")
    .WithUserName(postgresUser)
    .WithPassword(postgresPassword)
    .WithEnvironment("POSTGRES_DB", "db")
    .WithHostPort(5432);

var postgresdb = postgres.AddDatabase("db");

var webApi = builder.AddDockerfile("web-api", "../backend")
    .WithBuildArg("ENABLE_DRIFT", webApiEnableDrift)
    .WithEnvironment("DATABASE_URL", postgresdb.Resource.UriExpression)
    .WithEnvironment("HELIUS_API_KEY", heliusApiKey)
    .WithEnvironment("WALLET_PRIVATE_KEY", walletPrivateKey)
    .WithEnvironment("AI_GATEWAY_URL", aiGatewayUrl)
    .WithEnvironment("AI_GATEWAY_API_KEY", aiGatewayKey)
    .WithEnvironment("AI_GATEWAY_MODEL", aiGatewayModel)
    .WithHttpEndpoint(targetPort: 5150, port: 5150, env: "PORT", isProxied: false)
    .WithHttpHealthCheck("/")
    .WithExternalHttpEndpoints()
    .WithComputeEnvironment(compose);

webApi.WaitFor(postgresdb);

var migrations = builder.AddDockerfile("migrations", "../backend")
    .WithEntrypoint("backend-cli")
    .WithArgs("db", "migrate")
    .WithEnvironment("DATABASE_URL", postgresdb.Resource.UriExpression)
    .WithEnvironment("HELIUS_API_KEY", heliusApiKey)
    .WithEnvironment("WALLET_PRIVATE_KEY", walletPrivateKey)
    .WithComputeEnvironment(compose);

migrations.WaitFor(postgresdb);

var frontend = builder.AddDockerfile("frontend", "../frontend")
    .WithBuildArg("NEXT_PUBLIC_TRADING_API_URL", nextPublicTradingApiUrl)
    .WithHttpEndpoint(targetPort: 80, port: 80, env: "PORT", isProxied: false)
    .WithHttpHealthCheck("/")
    .WithExternalHttpEndpoints()
    .WithEnvironment("HOSTNAME", "0.0.0.0")
    .WithEnvironment("NEXT_PUBLIC_TRADING_API_URL", nextPublicTradingApiUrl)
    .WithEnvironment("WEB_API_HTTP", webApi.GetEndpoint("http"))
    .WithComputeEnvironment(compose);

frontend.WaitFor(webApi);

builder.Build().Run();
