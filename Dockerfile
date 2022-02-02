FROM rust:latest AS builder
MAINTAINER Kyle Cotton <kylecottonkc@gmail.com>
WORKDIR /usr/src/search

COPY . .
RUN cargo build --release

FROM gcr.io/distroless/cc-debian10
MAINTAINER Kyle Cotton <kylecottonkc@gmail.com>
EXPOSE 8000
COPY --from=builder /usr/src/search/target/release/search /workspace/search

# ENV SEARCH_PORT=8000
# ENV SEARCH_IP=0.0.0.0
# EXPOSE ${SEARCH_PORT}

CMD ["./workspace/search"]