IMAGE_NAME=wiki_search_api
IMAGE_VERSION=1.1.0

GRPC_PORT=50051
export SQLX_OFFLINE=true
export DATABASE_URL=postgresql://postgres:password@localhost:8001/only_graph
export SEARCH_PORT=8000
export RUST_LOG=info
export BACKEND=http://localhost:8000

run_img: #build_img
	docker run \
		-e SEARCH_PORT \
		-e SEARCH_IP=0.0.0.0 \
		-e GRPC_ADDRESS=0.0.0.0:${GRPC_PORT}\
		-e DATABASE_URL=${DATABASE_URL} \
		-e RUST_LOG=${RUST_LOG} \
		-e STATIC_DIR=./out \
		--rm -a stdin -a stdout -a stderr --network "host" ${IMAGE_NAME}:${IMAGE_VERSION} \

build_img:
	docker build . -t ${IMAGE_NAME}:${IMAGE_VERSION} \
		--build-arg BACKEND=${BACKEND}

# Note: This requires the sqlx-cli cargo extension to be installed
# This can be done using `cargo install sqlx-cli`
# After there has been a change to the database schema or queries
#	this command will need to be run again.
update-schema:
	cargo sqlx prepare -- --lib 

run:
	cd search && cargo run

build:
	cd search && cargo build --release 

test:
	cd search && cargo test --workspace

docs:
	cd search && cargo doc --open --no-deps
