FROM rust:1.58.1

WORKDIR /usr/src/search
COPY . .

# expose the api port
ENV SEARCH_PORT=8000
ENV SEARCH_IP=0.0.0.0
# this needs to be set before cargo build
ENV DATABASE_URL=postgresql://postgres:password@localhost:8001/only_graph

RUN cargo build --release



EXPOSE ${SEARCH_PORT}
CMD ["./target/release/search"]