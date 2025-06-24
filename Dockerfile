# ---- Stage 1: The Builder ----
# USE THE LATEST RUST IMAGE. It is based on Debian 12 "Bookworm"
# and has a new toolchain that understands `edition = "2024"`.
FROM rust:latest AS builder

WORKDIR /usr/src/hhz-bot

COPY . .

# This will now succeed because the toolchain is up-to-date.
# The binary will be linked against Bookworm's GLIBC.
RUN cargo build --release --bin hello_world


# ---- Stage 2: The Runtime ----
# CHANGE THIS LINE to use the corresponding modern OS base image.
FROM debian:bookworm-slim AS runtime

# Copy the compiled binary from the "builder" stage.
# Both stages use Bookworm, so there is no GLIBC mismatch.
COPY --from=builder /usr/src/hhz-bot/target/release/hello_world /usr/local/bin/hhz-bot

# Set the command to run your application
CMD ["hhz-bot"]