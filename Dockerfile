FROM rust:latest AS builder
MAINTAINER Kyle Cotton <kylecottonkc@gmail.com>
WORKDIR /usr/src/search

COPY . .
RUN cargo build --release

FROM gcr.io/distroless/cc-debian10
MAINTAINER Kyle Cotton <kylecottonkc@gmail.com>
EXPOSE 8000
EXPOSE 50051
COPY --from=builder /usr/src/search/target/release/search /workspace/search

CMD ["./workspace/search"]