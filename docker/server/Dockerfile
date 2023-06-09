

FROM    rust:1.70 as builder

COPY    . .
RUN     cargo build --release


FROM    debian:buster-slim
COPY    --from=builder ./target/release/ /app/release/
