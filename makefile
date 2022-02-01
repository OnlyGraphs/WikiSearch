IMAGE_NAME=wiki_search_api
IMAGE_VERSION=0.1

export DATABASE_URL=postgresql://postgres:password@localhost:8001/only_graph

run_img: build_img
	export SEARCH_PORT=8000 &&
	export SEARCH_IP=0.0.0.0 &&
	export GRPC_ADDRESS=0.0.0.0:50051
	docker run -p ${SEARCH_PORT}:${SEARCH_PORT} --rm -a stdin -a stdout ${IMAGE_NAME}:${IMAGE_VERSION}

build_img:
	docker build . -t ${IMAGE_NAME}:${IMAGE_VERSION}

run:
	cargo run 

build:
	cargo build --release

test:
	cargo test

docs:
	cargo doc --open --no-deps
