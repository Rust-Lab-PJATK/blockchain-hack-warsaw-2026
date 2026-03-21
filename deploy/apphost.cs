#:sdk Aspire.AppHost.Sdk@13.1.3
#:package Aspire.Hosting.PostgreSQL@*
#:package Aspire.Hosting.JavaScript@13.1.3
#:package Aspire.Hosting.Docker@13.1.3-preview.1.26166.8 

var builder = DistributedApplication.CreateBuilder(args);
var compose = builder.AddDockerComposeEnvironment("compose");

var postgresUser = builder.AddParameter("postgres-user", "loco");
var postgresPassword = builder.AddParameter("postgres-password", "loco", secret: true);

var postgres = builder.AddPostgres("postgres")
    .WithUserName(postgresUser)
    .WithPassword(postgresPassword)
    .WithEnvironment("POSTGRES_DB", "db")
    .WithHostPort(5432);

var postgresdb = postgres.AddDatabase("db");

var webApi = builder.AddDockerfile("web-api", "../backend")
    .WithEnvironment("DATABASE_URL", postgresdb.Resource.UriExpression)
    .WithHttpEndpoint(targetPort: 5150, port: 5150, env: "PORT", isProxied: false)
    .WithExternalHttpEndpoints()
    .WithComputeEnvironment(compose);

webApi.WaitFor(postgresdb);

var frontend = builder.AddDockerfile("frontend", "../frontend")
    .WithHttpEndpoint(targetPort: 3000, port: 3000, env: "PORT")
    .WithExternalHttpEndpoints()
    .WithEnvironment("HOSTNAME", "0.0.0.0")
    .WithEnvironment("WEB_API_HTTP", webApi.GetEndpoint("http"))
    .WithComputeEnvironment(compose);

frontend.WaitFor(webApi);

builder.Build().Run();
