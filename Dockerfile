# syntax=docker/dockerfile:1

FROM rust
COPY . /app/src
WORKDIR /app/src
RUN cargo install --path .
CMD kersd
