FROM rust:latest
WORKDIR /usr/src/server

EXPOSE 8080

COPY ./target/release/key-value-store-server .
ENTRYPOINT ["./key-value-store-server"]
