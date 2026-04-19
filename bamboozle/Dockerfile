# Stage 1: Build
# rust:alpine uses musl libc — produces a fully static binary, enabling FROM scratch.
FROM rust:alpine AS builder
RUN apk add --no-cache musl-dev curl
WORKDIR /app

# Pre-build dependencies in a separate layer so they cache on source-only changes.
COPY Cargo.toml Cargo.lock ./
RUN mkdir src && echo "fn main(){}" > src/main.rs \
    && cargo build --release \
    && rm -f target/release/.fingerprint/bamboozle-*/invoked.timestamp

# Build the real source
COPY src ./src
RUN touch src/main.rs && cargo build --release

# Stage 2: Zero-byte base — works because the musl binary is fully static.
FROM scratch
COPY --from=builder /app/target/release/bamboozle /bamboozle
EXPOSE 8080 9090
ENTRYPOINT ["/bamboozle"]
