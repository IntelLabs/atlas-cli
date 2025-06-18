FROM rust:1.87 as builder

WORKDIR /usr/src/app
COPY . .
RUN cargo build --release

FROM debian:bookworm-slim

RUN apt-get update && \
    apt-get install -y --no-install-recommends \
    ca-certificates \
    libssl-dev \
    && rm -rf /var/lib/apt/lists/*

COPY --from=builder /usr/src/app/target/release/c2pa_storage_service /usr/local/bin/

EXPOSE 8080
CMD ["c2pa_storage_service"]
