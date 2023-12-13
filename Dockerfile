FROM rust:1.74-alpine

RUN mkdir -p /actix-web/www
WORKDIR /actix-web/www

COPY Cargo.lock Cargo.lock
COPY Cargo.toml Cargo.toml
COPY ./src ./src
RUN cargo install --path .

CMD ["hello-actix"]