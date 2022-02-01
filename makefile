IMAGE_NAME="wiki_search_api"
IMAGE_VERSION="0.1"
SEARCH_PORT=8000
SEARCH_IP=0.0.0.0

run_img: build_img
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
