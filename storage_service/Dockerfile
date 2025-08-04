FROM rust:1.86 as builder

WORKDIR /usr/src/app
COPY . .
RUN cargo build --release

FROM debian:trixie-slim

RUN DEBIAN_FRONTEND=noninteractive && apt-get update && apt-get upgrade --assume-yes 

RUN apt-get install -y --no-install-recommends \
    ca-certificates \
    libssl-dev \
    && rm -rf /var/lib/apt/lists/*

COPY --from=builder /usr/src/app/target/release/c2pa_storage_service /usr/local/bin/


EXPOSE 8080
CMD ["c2pa_storage_service"]
