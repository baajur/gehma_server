FROM rust:1.38 as builder
RUN apt-get update
RUN apt-get install -y openssl postgresql postgresql-contrib

WORKDIR /gehma/diesel_dir
RUN cargo install diesel_cli --no-default-features --features postgres --root /gehma/diesel_dir

WORKDIR /gehma
RUN USER=root cargo new --bin gehma
COPY ./core ./core
COPY ./Cargo.lock ./Cargo.lock
COPY ./Cargo.toml ./Cargo.toml
COPY ./src ./src
RUN cargo build --release

FROM ubuntu:latest
WORKDIR /gehma
RUN apt-get update
RUN apt-get install -y openssl postgresql-client ca-certificates

COPY ./migrations ./migrations
COPY --from=builder /gehma/diesel_dir/bin/diesel . 
COPY --from=builder /gehma/target/release/sprechstunde /gehma/sprechstunde

CMD ["./diesel migration run"]
CMD ["/gehma/sprechstunde"]
