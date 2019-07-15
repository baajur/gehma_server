FROM rust:1.31

RUN sudo apt-get install openssl libssl-dev

WORKDIR /usr/src/gehma

COPY . .

RUN cargo install --path .

CMD["./target/debug/sprechstunde"]
