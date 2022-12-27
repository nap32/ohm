# About

Ω Ω Ω Ω Ω Ω Ω Ω Ω Ω

Ohm is an intercepting proxy designed to passively parse request-response pairs into a common format and update/insert to a database.

Ohm is written in rust using the powerful tokio asynchronous runtime engine and leveraging hyper as an HTTP library.

As a result, Ohm is a fast-and-safe asynchronous HTTP / HTTPS intercepting proxy.

# Setup

Ohm requires a bit of setup out-of-the-box.

### Generate CA.

First, you'll have to generate a certificate authority in the correct format to use for TLS interception.
It is used to self-sign dynamically generated X509 certificates so that your browser believes it's talking to the intended server.
Ohm actually uses the logic in `crate::service::ca` to break-an-inspect traffic; maintaining two TLS tunnels.
This behavior is common in all intercepting proxies as a requirement for HTTPS traffic interception - don't forget to install the CA in the browser.

### Setup Database.

Second, you need to have a running database to store the traffic parsed from the browser - such as a docker container running the `mongo:latest` image.
It is highly recommended to specify the local interface if you're running the docker container on your local testing laptop to prevent exposing secrets.

### Configure Ohm.

Third, you'll need to modify `config.yaml` to specify the correct details relating to your certificate locations and database instance.

# Usage

Once Ohm is up-and-running, the proxy satisfies a powerful means to ingest browser traffic as persistent records into a data store.

Without too much effort, using the database solution's tooling provides a local logging mechanism.
By leveraging existing knowledge of queries, you can quickly answer questions such as:
        - "What routes have I enumerated for this API?"
        - "How many services do I know about?"
        - "Do I know of any applications using this potentially problematic header?"

Through consistent use, you'll build up more complete information over time without needing to maintain the traffic in notes or navigate intercepting proxy traffic history.

This is not the only value that Ohm can provide, as the database's resulting collection of traffic records provides a flexible interface to streamline automation efforts.
Ohm is designed to avoid introducing undesirable domain-specific languages or interface definition languages and instead offer the flexibility for its users to leverage database solutions.
You can tailor your tooling to your specific use-case or workload - potential ideas to consider include:
        - Using event triggers native to the database enables filtering of traffic containing plaintext credentials sent to your identity provider as logins occur.
        - Using event triggers to filter traffic in response to a new push to the record's traffic array can allow you to drop documents in the collection you don't care about - like images, .js/.ts, google analytics, etc.
        - Using event triggers, you can de-duplicate records that are similar by means of path variables to condense records with user emails, GUIDs/UUIDs, or numbers to an abstracted generic representation.
        - Using scheduled triggers, you can periodically port scan services or replay requests to monitor API changes over time.
        - You can implement extensions to the 'record' document to provide a field that functions as an array of labels/tags/URLs to organize reporting or couple endpoints with JIRA tickets.
        - You can implement extensions to the 'record' document to provide a field that couples note-taking directly with the information it concerns.
        - You can implement extensions to the 'record' document to provide an embedded document field containing the relevant information to automate generating a new token via OAuth2 through a known identity provider.

Rather than include solutions to some or all of the suggestions above in rigid application-level logic, Ohm is a one trick pony -
Listen to and record 'all the things' into a format that's reused across the rest of a user's/team's ecosystem.
This avoids enforcing opinions on how to use the traffic and instead offers the user the opportunity to come up with their own solutions.

# Thanks

Ohm is inspired by or has benefitted from the ideas or code contained in the following projects:
- https://github.com/mitmproxy/mitmproxy
- https://github.com/omjadas/hudsucker
- https://github.com/
- https://github.com/tokio-rs/tokio
- https://github.com/hyperium/hyper

In addition, many other libaries enable Ohm to function. Check out the `Cargo.toml` for a complete list of dependencies.
Thank you!
