FROM rust:1.58.1

WORKDIR /usr/src/search
COPY . .

# build in /usr/src/search
RUN cargo build --release


# expose the api port
ENV SEARCH_PORT=80
ENV SEARCH_IP=0.0.0.0

EXPOSE ${SEARCH_PORT}
CMD ["./target/release/search"]