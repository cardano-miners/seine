FROM lukemathwalker/cargo-chef:latest-rust-1 AS chef

WORKDIR /app

FROM chef AS planner

COPY . .

RUN cargo chef prepare --recipe-path recipe.json

FROM chef AS builder

COPY --from=planner /app/recipe.json recipe.json

# Build dependencies - this is the caching Docker layer!
RUN cargo chef cook --release --recipe-path recipe.json

# Build application
COPY . .

RUN cargo build --release --bin seine

# We do not need the Rust toolchain to run the binary!
FROM debian:bookworm-slim AS runtime

# Update and install necessary packages
RUN apt-get update && \
    apt-get upgrade -y && \
    apt-get install -y \
    pkg-config \
    libssl-dev \
    ca-certificates && \
    rm -rf /var/lib/apt/lists/*

# Update CA certificates
RUN update-ca-certificates

WORKDIR /app

COPY --from=builder /app/target/release/seine /usr/local/bin

ENTRYPOINT ["/usr/local/bin/seine"]
