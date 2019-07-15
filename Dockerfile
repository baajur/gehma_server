FROM rust:1.35

RUN apt-get install openssl libssl-dev

WORKDIR /usr/src/gehma

RUN cargo install diesel_cli

COPY . .

RUN cargo build

CMD ["/usr/local/cargo/bin/diesel migration run"]
CMD ["/usr/src/gehma/target/debug/sprechstunde"]
