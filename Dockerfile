FROM nobodyxu/rust-and-clang as builder
WORKDIR /usr/src/pasty
COPY . .
RUN cargo install --path .

FROM alpine:latest
COPY --from=builder /usr/local/cargo/bin/pasty /usr/local/bin/pasty
ENTRYPOINT ["/usr/local/bin/pasty"]