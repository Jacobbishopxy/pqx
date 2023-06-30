# author:	Jacob Xie
# @date:	2023/06/22 09:15:31 Thursday
# @brief:

ARG     RUST_VERSION

# PLANNER
FROM    rust:${RUST_VERSION} as planner
WORKDIR /app
RUN     cargo install cargo-chef
COPY    . .
RUN     cargo chef prepare --recipe-path recipe.json

###################################################################################################
# CACHER
FROM    rust:${RUST_VERSION} as cacher
WORKDIR /app
RUN     cargo install cargo-chef
COPY    --from=planner /app/recipe.json recipe.json
RUN     cargo chef cook --release --recipe-path recipe.json

###################################################################################################
# BUILDER
FROM    rust:${RUST_VERSION} as builder
WORKDIR /app
COPY    . .
COPY    --from=cacher /app/target target
RUN     cargo build --release

###################################################################################################
# APP
FROM    rust:${RUST_VERSION}
WORKDIR /app
# RUN     apt-get update && apt-get install -y libssl-dev build-essential
# RUN     cargo install cargo-make
COPY    --from=builder /app/target/release/ /usr/local/bin

