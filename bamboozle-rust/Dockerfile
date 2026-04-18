# Stage 1: Build
# Use the full Rust image (not slim) — it includes gcc, pkg-config, and other
# native build tools required by some dependency build scripts.
# The final runtime stage stays minimal, so this doesn't affect image size.
FROM rust AS builder
WORKDIR /app

# Pre-build dependencies in a separate layer so they are cached on source-only changes.
COPY Cargo.toml Cargo.lock ./
RUN mkdir src && echo "fn main(){}" > src/main.rs \
    && cargo build --release \
    && rm -f src/main.rs

# Now build the real source
COPY src ./src
RUN touch src/main.rs && cargo build --release

# Stage 2: Minimal runtime image
FROM debian:bookworm-slim AS runtime
RUN apt-get update \
    && apt-get install -y ca-certificates \
    && rm -rf /var/lib/apt/lists/*

COPY --from=builder /app/target/release/bamboozle /usr/local/bin/bamboozle

EXPOSE 8080 9090

ENTRYPOINT ["/usr/local/bin/bamboozle"]
