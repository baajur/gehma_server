FROM rust:1.38 as builder
RUN apt-get update
RUN apt-get install -y openssl postgresql postgresql-contrib

WORKDIR /gehma
#RUN USER=root cargo new --bin gehma
RUN mkdir core
COPY ./Cargo.lock ./Cargo.lock
COPY ./Cargo.toml ./Cargo.toml
COPY ./core/Cargo.toml ./core/Cargo.toml

RUN mkdir ~/.cargo
RUN cargo vendor > ~/.cargo/config

COPY ./core core
COPY ./src src
RUN cargo build --release

FROM ubuntu:latest
WORKDIR /gehma
RUN apt-get update
RUN apt-get install -y openssl postgresql-client ca-certificates

COPY ./migrations ./migrations
COPY --from=builder /gehma/target/release/sprechstunde /gehma/sprechstunde

CMD ["/gehma/sprechstunde"]
