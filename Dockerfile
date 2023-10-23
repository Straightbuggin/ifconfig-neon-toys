# Stage 1: Build the Rust application
FROM rust:latest as builder

WORKDIR /usr/src/ifconfig-neon-toys
COPY . .

RUN cargo build --release

# Stage 2: Create a minimal image to run the compiled binary
FROM debian:bookworm-slim

# Install OpenSSL
RUN apt-get update && \
    apt-get install -y openssl libssl-dev && \
    rm -rf /var/lib/apt/lists/*

# Create a new directory for the app and set it as the working directory
WORKDIR /app/ifconfig-neon-toys

COPY --from=builder /usr/src/ifconfig-neon-toys/target/release/ifconfig-neon-toys /usr/local/bin/ifconfig-neon-toys
COPY --from=builder /usr/src/ifconfig-neon-toys/response.html .


EXPOSE 8080

CMD ["ifconfig-neon-toys"]

