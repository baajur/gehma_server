FROM rust:1.40 as builder
RUN apt-get update
RUN apt-get install -y openssl postgresql postgresql-contrib

WORKDIR /gehma
#RUN USER=root cargo new --bin gehma
COPY . .
RUN cd gehma_server && cargo build --release

FROM ubuntu:latest
WORKDIR /gehma
RUN apt-get update
RUN apt-get install -y openssl postgresql-client ca-certificates

COPY ./migrations ./migrations
COPY --from=builder /gehma/target/release/sprechstunde /gehma/sprechstunde

CMD ["/gehma/sprechstunde"]
