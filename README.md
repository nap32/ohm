# About

Ω Ω Ω

\
Ohm is a fast and memory-safe asynchronous HTTP / HTTPS intercepting proxy. \
Ohm is designed to passively parse request-response pairs and write the information to a database.\
The intention is to expose local browser traffic in a database to support automation and gather information.\
The hope is that Ohm is flexible enough to support a variety of use-cases.

### Current Issues / Next Steps

What stands between Ohm and public release?

- Need to revamp redis and postgres.
- Need to add record to database interface.
- Need to support using other databases (mongo currently hardcoded).
- Need a lot of testing coverage.
- Need to gracefully handle websockets.
- Need to handle identifying other half of AuthInfo from usage.
- Need to handle aud and scopes in AuthInfo.
- Need to handle all the .unwrap()'s as it isn't good.
- Need to refactor the way request and response clone traffic - seek better solution.
- Need to actually make a good README.md
- Chaining proxies has Ohm listen over `http://` instead of `https://` - need to investigate.

### Features

- HTTP/HTTPS Intercepting Proxy
- Supports multiple major databases - Mongo, Postgres, and Redis; store your data in your preferred format.
- Works in tandem with existing intercepting proxies - chain multiple proxies together.
- Leverage your preferred database solution's feature-rich ecosystem to manage your traffic history.
- Designed as a passive proxy - processing records spawned as a seperate thread/task and traffic returned to the user without synchronous blocking.
- Designed in a memory-safe way.
- Ohm is fast.

### Design Decisions

##### Decompression of response bodies happens by default.
While the intention was to store traffic as-is to keep it usage flexible,
you can't store encoded bodies to the datastore and also effectively search the contents.

##### Application-level mechanism for filtering.
The original intention was to leverage datastore event triggers to filter traffic.\
The hope was to avoid defining an interface whose syntax needed memorized.\
There are a couple of issues with this approach -
1. Not every datastore has event triggers or is capable of filtering traffic on write.
2. A multi-user datastore would leak issuer traffic and session tokens.

To compromise, Ohm sets up a minimal filtering chain interface that can be extended in configuration or source.\
This enables the ability to configure behavior of the filters without recompiling from source.\
Where more advanced filtering behavior is needed, you can write another function or use downstream mechanisms like event triggers.

# Setup

Ohm requires a bit of setup out-of-the-box.

### Generate CA.

First, you'll have to generate a certificate authority in the correct format to use for TLS interception.\
It is used to self-sign dynamically generated X509 certificates so that your browser believes it's talking to the intended server.\
Ohm actually uses the logic in `crate::service::ca` to break-an-inspect traffic; maintaining two TLS tunnels.\
This behavior is common in all intercepting proxies as a requirement for HTTPS traffic interception - don't forget to install the CA in the browser.

```
openssl genrsa -des3 -out ohm.key 2048
openssl req -x509 -new -nodes -key ohm.key -sha256 -days 1825 -out ohm.pem
```

### Setup Database.

Second, you need to have a running database to store the traffic parsed from the browser - such as a docker container running the `mongo:latest` image.\
It is highly recommended to specify the local interface if you're running the docker container on your local testing laptop to prevent exposing secrets.

```
docker pull image mongo
docker run --name ohm-mongo -d -p 127.0.0.1:27017:27017 mongo
```

### Configure Ohm.

Third, you'll need to modify `config.yaml` to specify the correct details relating to your certificate locations and database instance.

# Usage
\
Once Ohm is up-and-running, the proxy passively ingests browser traffic into the configured datastore.\
\
Using the database solution's tooling provides a local logging mechanism out-of-the-box.\
By querying your traffic records, you can quickly answer questions such as:

    * "What endpoints/routes have I enumerated for this API?"
    * "How many unique services do I know about on this subdomain?"
    * "Do I know of any applications using this potentially problematic header?"
    * "How do I make this POST request again - what form fields do I need?"

Through consistent use, you'll build up more complete information over time without needing to maintain the traffic in notes or navigate intercepting proxy traffic history.\
The result is a collection of traffic records that provide a flexible interface to streamline automation efforts.\
Ohm is designed to avoid introducing undesirable domain-specific languages or interface definition languages and instead offer the flexibility for its users to leverage database solutions.\
You can tailor your tooling to your specific use-case or workload - potential ideas to consider include:

    * Using event triggers native to the database enables filtering of traffic containing plaintext credentials sent to your identity provider as logins occur.\
    * Using event triggers to filter traffic in response to a new push to the record's traffic array can allow you to drop documents in the collection you don't care about - like images, .js/.ts, google analytics, etc.\
    * Using event triggers, you can de-duplicate records that are similar by means of path variables to condense records with user emails, GUIDs/UUIDs, or numbers to an abstracted generic representation.\
    * Using scheduled triggers, you can periodically port scan services or replay requests to monitor API changes over time.\
    * You can implement extensions to the 'record' document to provide a field that functions as an array of labels/tags/URLs to organize reporting or couple endpoints with JIRA tickets.\
    * You can implement extensions to the 'record' document to provide a field that couples note-taking directly with the information it concerns.\
    * You can implement extensions to the 'record' document to provide an embedded document field containing the relevant information to automate generating a new token via OAuth2 through a known identity provider.\

Rather than include solutions to some or all of the suggestions above in rigid application-level logic, Ohm is a one trick pony -\
Listen to and record 'all the things' into a format that's reused across the rest of a user's/team's ecosystem.\
This avoids enforcing opinions on how to use the traffic and instead offers the user the opportunity to come up with their own solutions.

### Quick Start.

Follow setup in section above.
Make sure the container for the database is started.

```
docker exec -it ohm-mongo mongosh

cargo run

use ohm
db.traffic.find({"host":{"$not":{"$regex":"xyz","$options":"i"}}})
db.authinfo.distinct("client_id")
exit
```

# Chaining Proxies w/ Ohm

Ohm is capable of working alongside other proxy solutions.\
While the setup might be specific to the individual proxy(chain), common intercepting proxy examples are detailed below:

### mitmproxy

Configure the browser to point to mitmproxy as you normally would.\
Update the configuration file of Ohm to ensure both proxies don't attempt to listen to the same port.\
Run mitmproxy with an upstream flag:

```
mitmproxy --mode upstream:http://127.0.0.1:8085
```

If you use an `https://` scheme instead of `http://`, mitmproxy will complain that the upstream server doesn't speak TLS.\
Be sure to restrict to the local interface and appropriately lock down.\
Consider namespaces or putting everything behind a docker network so only Mitmproxy and Ohm are on a LAN.

# Warning!

Ohm does not prevent the user from misconfiguring or exposing secrets during usage.\
Several mistakes can be made in setup that result in a security issue:

    1. If you're testing the tool and stand up a database container locally, make sure you bind it to the local interface to prevent yourself from offering traffic to your LAN or WAN.
    2. If you login through a site not listed in the configuration file as an identity provider, the username and password will be logged to the database.
    3. If you don't setup your datastore with authentication, you're hosting traffic containing session tokens to anyone who can interface with the datastore.
    4. It would be wise to encrypt the datastore at rest to prevent leaking sensitive information - credentials, PII, internal-only services.

The list above is not exhaustive.\
The user is responsible for securing their own local environment.\
Ohm and it's maintainer(s) accept no responsibility for issues caused through its use.

# Thanks

Ohm is inspired by or has benefitted from the ideas or code contained in the following projects:\

    * https://github.com/mitmproxy/mitmproxy
    * https://github.com/omjadas/hudsucker
    * https://github.com/tokio-rs/tokio
    * https://github.com/hyperium/hyper

Check out the `Cargo.toml` for a complete list of dependencies.\
Thank you!
