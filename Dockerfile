# syntax=docker/dockerfile:1

FROM rust
COPY . /src
WORKDIR /src
RUN cargo install --path .
CMD kersd

