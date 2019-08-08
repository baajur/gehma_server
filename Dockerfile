FROM rust:1.35

RUN apt-get update -y && apt-get upgrade -y
RUN apt-get install -y openssl libssl-dev make gcc

WORKDIR /usr/src/gehma

RUN cargo install diesel_cli --no-default-features --features postgres

COPY . .

RUN cargo build

ENV DATABASE_URL
ENV PORT
ENV DEBUG
ENV BINDING_ADDR
ENV FCM_TOKEN
ENV PRIVATE_KEY_PATH
ENV CERT_PATH

CMD ["/usr/local/cargo/bin/diesel migration run"]
CMD ["/usr/src/gehma/target/debug/sprechstunde"]
