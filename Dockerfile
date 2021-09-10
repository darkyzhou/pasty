FROM nobodyxu/rust-and-clang as builder
WORKDIR /usr/src/pasty
COPY . .
RUN cargo install --path .

FROM debian:10-slim
COPY --from=builder /usr/local/cargo/bin/pasty /usr/local/bin/pasty
COPY ./Rocket.toml /
WORKDIR /
CMD ["pasty"]