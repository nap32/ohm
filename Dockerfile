FROM rust:1.61.0 as builder
WORKDIR /usr/src/ohm
COPY . .
RUN cargo install --path .
 
FROM debian:bullseye-slim
WORKDIR /usr/src/ohm
RUN apt-get update && apt-get install -y libssl1.1 libssl-dev libc6 && rm -rf /var/lib/apt/lists/*
COPY --from=builder /usr/local/cargo/bin/ohm /usr/local/bin/ohm
COPY ./config/ /etc/ohm/config/
CMD ["ohm", "/etc/ohm/config/config.toml", "&"]
