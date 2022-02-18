IMAGE_NAME=wiki_search_api
IMAGE_VERSION=0.1

export SQLX_OFFLINE=true
export DATABASE_URL=postgresql://postgres:password@localhost:8001/only_graph
export SEARCH_PORT=8000
export GRPC_PORT=50051
export RUST_LOG=debug

run_img: #build_img
	docker run -p ${SEARCH_PORT}:8000 \
		-p ${GRPC_PORT}:50051 \
		-e SEARCH_PORT \
		-e SEARCH_IP=0.0.0.0 \
		-e GRPC_ADDRESS=0.0.0.0:${GRPC_PORT}\
		-e DATABASE_URL=${DATABASE_URL} \
		-e RUST_LOG=${RUST_LOG} \
		--rm -a stdin -a stdout ${IMAGE_NAME}:${IMAGE_VERSION} \

build_img:
	docker build . -t ${IMAGE_NAME}:${IMAGE_VERSION}

# Note: This requires the sqlx-cli cargo extension to be installed
# This can be done using `cargo install sqlx-cli`
# After there has been a change to the database schema or queries
#	this command will need to be run again.
update-schema:
	cargo sqlx prepare

run:
	cargo run

build:
	cargo build --release

test:
	cargo test

docs:
	cargo doc --open --no-deps
