# Ω Ohm
## About

- A HTTP / HTTPS intercepting proxy that stores traffic in a database.
- Built as a passive proxy with a focus on fast performance, safe design, and flexible usage.
- Increase observability into your traffic and support automation efforts by passively recording test cases you generate.
- Capable of supporting usage locally client-side to record browser traffic or server-side for selective traffic logging.
- A filtering chain provides powerful capabilities to handle decision making on traffic ingestion, safely handling identity provider traffic, or regex string replacement.
- The same filtering chain decodes gzip, brotli, or deflate encodings to make the recorded traffic easy to work with for database searches or automation.

## Setup

- Ohm requires a bit of setup out-of-the-box.
- You need to generate a CA and install it in your local brower.
- This is needed to generate and sign trusted certificates trusted by the browser to break-and-inspect HTTPS traffic.
- You'll also need a database to store traffic - the easiest way for testing is to use a docker container running `mongo:latest`.
- It is highly recommended to specify the local interface if you're running the docker container on your local testing laptop to prevent exposing secrets.
- Make sure to modify `config.yaml` to specify the correct details relating to your certificate locations and database instance.

### Quick Start.

Generate a certificate and install it in your local web browser.

```
openssl genrsa -des3 -out ohm.key 2048
openssl req -x509 -new -nodes -key ohm.key -sha256 -days 1825 -out ohm.pem

```

Run a docker image.

```
docker pull mongo
docker run --name mongo -d -p 127.0.0.1:27017:27017 mongo
```

Set up the config.yaml \
Run Ohm.

```
cargo run
```

Search your traffic.

```
docker exec -it ohm-mongo mongosh

use ohm
db.traffic.find({"host":{"$not":{"$regex":"xyz","$options":"i"}}})
db.authinfo.distinct("client_id")
exit
```

## Warning!

Ohm does not prevent the user from misconfiguring or exposing secrets during usage.\
Several mistakes can be made in setup that result in a security issue:
        
1. If you're testing the tool and stand up a database container locally, make sure you bind it to the local interface (and not, for example, 0.0.0.0).
2. If you login through a site not listed in the configuration file as an identity provider, the username and password will be logged to the database.
3. If you don't setup your datastore with authentication, you're hosting traffic containing session tokens to anyone who can interface with the datastore.
4. It would be wise to encrypt the datastore at rest to prevent leaking sensitive information - credentials, PII, internal-only services.

The list above is not exhaustive.\
The user is responsible for securing their own local environment.\
Ohm and it's maintainer(s) accept no responsibility for issues caused through its use.

## Thanks

Ohm is inspired by or has benefitted from the ideas or code contained in the following projects:
* https://github.com/omjadas/hudsucker
* https://github.com/mitmproxy/mitmproxy

## Design Decisions and Trade-Offs.

### Decompression of response bodies happens by default.
While the intention was to store traffic as-is to keep it usage flexible,\
you can't store encoded bodies to the datastore and also effectively search the contents.\
The filtering chain is set up to decode text encodings (gzip, brotli, deflate) by default.

### Application-level mechanism for filtering.
The original intention was to leverage datastore event triggers to filter traffic.\
The hope was to avoid defining an interface whose syntax needed memorized.\
There are a couple of issues with this approach -
1. Not every datastore has event triggers or is capable of filtering traffic on write.
2. A multi-user datastore could leak issuer traffic and session tokens.

To compromise, Ohm sets up a minimal filtering chain interface that can be extended in configuration or source.\
This enables limited configuration of filtering behavior without recompiling from source.\
Where more advanced filtering behavior is needed, you can write another function, edit the filtering chain, or use downstream mechanisms like event triggers.

### Chaining Proxies w/ Ohm

Ohm is capable of working alongside other proxy solutions.\
While the setup might be specific to the individual proxy(chain), common intercepting proxy examples are detailed below:

#### mitmproxy

Configure the browser to point to mitmproxy as you normally would.\
Update the configuration file of Ohm to ensure both proxies don't attempt to listen to the same port.\
Run mitmproxy with an upstream flag:

```
mitmproxy --mode upstream:http://127.0.0.1:8085
```

If you use an `https://` scheme instead of `http://`, mitmproxy will complain that the upstream server doesn't speak TLS.\
Be sure to restrict to the local interface and appropriately lock down.\
Consider namespaces or putting everything behind a docker network so only Mitmproxy and Ohm are on a LAN.
