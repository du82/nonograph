# Build stage
FROM rust:1.82 as builder

WORKDIR /app

# Copy manifests
COPY Cargo.toml ./

# Copy source code
COPY src ./src

# Build the application
RUN cargo build --release

# Runtime stage
FROM debian:bookworm-slim

# Install runtime dependencies
RUN apt-get update && \
    apt-get install -y ca-certificates && \
    rm -rf /var/lib/apt/lists/*

WORKDIR /app

# Copy the built binary
COPY --from=builder /app/target/release/nonograph /app/nonograph

# Copy templates and config
COPY templates ./templates
COPY Config.toml ./Config.toml

# Create content directory
RUN mkdir -p content

EXPOSE 3000

CMD ["./nonograph"]
