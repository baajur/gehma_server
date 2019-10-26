FROM rust:1.38
RUN apt-get update
RUN apt-get install -y openssl postgresql postgresql-contrib

RUN cargo install diesel_cli --no-default-features --features postgres

RUN USER=root cargo new --bin gehma
WORKDIR /gehma
COPY ./core ./core
COPY ./Cargo.lock ./Cargo.lock
COPY ./Cargo.toml ./Cargo.toml
COPY ./src ./src
RUN cargo build --release

COPY ./migrations ./migrations

CMD ["diesel migration run"]
CMD ["/gehma/target/release/sprechstunde"]
