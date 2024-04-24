# STAGE 1 - PLANNING
# Cargo chef stable release matching our rustc version
FROM lukemathwalker/cargo-chef:latest-rust-1.77 as chef
WORKDIR /app
RUN apt update && apt install lld clang -y

# Prepare our dependencies
FROM chef as planner
COPY . .
# Compute a lock-like file for our project
RUN cargo chef prepare  --recipe-path recipe.json

# STAGE 2 - BUILD DEPENDENCIES
# Build our project dependencies, not our application!
FROM chef as cacher
COPY --from=planner /app/recipe.json recipe.json
RUN cargo chef cook --release --recipe-path recipe.json

# STAGE 3 - BUILD PROJECT USING DEPENDENIES FROM STAGE 2
# We use the Rust stable release matching our version as base image
FROM rust:1.77 as builder
# Required by sqlx to read queries from .sqlx. Only used for docker build
ENV SQLX_OFFLINE true
# To ensure that correct host is used
ENV APP_ENVIRONMENT production
WORKDIR /app
COPY . .
# copy dependencies built in the previous stage
COPY --from=cacher /app/target target
COPY --from=cacher /usr/local/cargo /usr/local/cargo

# Build our project
RUN cargo build --release --bin newsletter_app

# STAGE 4 - COPY COMPILED PROJECT AND CONFIGURATION INTO MINIMALISTIC IMAGE
FROM ubuntu:22.04 AS runtime
ENV APP_ENVIRONMENT production
WORKDIR /app
# Install OpenSSL - it is dynamically linked by some of our dependencies
# Install ca-certificates - it is needed to verify TLS certificates
# when establishing HTTPS connections
RUN apt-get update -y \
    && apt-get install -y --no-install-recommends openssl ca-certificates \
# Clean up
    && apt-get autoremove -y \
    && apt-get clean -y \
    && rm -rf /var/lib/apt/lists/*
COPY --from=builder /app/target/release/newsletter_app newsletter_app
COPY configuration configuration
# When `docker run` is executed, launch the binary!
ENTRYPOINT ["./newsletter_app"]