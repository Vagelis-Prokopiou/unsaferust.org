FROM rust:1.67-slim-bullseye AS chef

WORKDIR /app

RUN cargo install cargo-watch --locked \
    && cargo install cargo-chef --locked \
    && rustup component add clippy \
    && apt-get update \
    && apt-get install -y \
    git \
    grep \
    cloc \
    pkg-config \
    openssl \
    libssl-dev \
    && rm -rf /var/lib/apt/lists/*

# This stage is common for both dev and prod.
FROM chef AS planner
COPY . .
RUN cargo chef prepare --recipe-path recipe.json

# This is the profile that we run for development.
FROM chef AS dev
COPY --from=planner /app/recipe.json recipe.json
# Build dependencies - this is the caching Docker layer!
RUN cargo chef cook --recipe-path recipe.json
# Build application
COPY . .
CMD cargo watch --clear --quiet -x test -x clippy -x run
