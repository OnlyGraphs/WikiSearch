IMAGE_NAME=wiki_search_api
IMAGE_VERSION=1.1.0
GRPC_PORT=50051
ARGS=''
export SQLX_OFFLINE=true
export DATABASE_URL=postgresql://postgres:password@localhost:8001/only_graph
export SEARCH_PORT=8000
export RUST_LOG=debug,sqlx=off
export BACKEND=http://localhost:8000
export RUST_BACKTRACE=1
export CACHE_SIZE=500000
export SQL_MAX_CLIENTS=100
run_img: #build_img
	docker run \
		-e SEARCH_PORT \
		-e SEARCH_IP=0.0.0.0 \
		-e GRPC_ADDRESS=0.0.0.0:${GRPC_PORT}\
		-e DATABASE_URL=${DATABASE_URL} \
		-e RUST_LOG=${RUST_LOG} \
		-e STATIC_DIR=./out \
		-e CACHE_SIZE=${CACHE_SIZE} \
		-e SQL_MAX_CLIENTS=${SQL_MAX_CLIENTS} \
		--rm -a stdin -a stdout -a stderr --network "host" ${IMAGE_NAME}:${IMAGE_VERSION} \

build_img:
	docker build . -t ${IMAGE_NAME}:${IMAGE_VERSION} \
		--build-arg BACKEND=${BACKEND}

# Note: This requires the sqlx-cli cargo extension to be installed
# This can be done using `cargo install sqlx-cli`
# After there has been a change to the database schema or queries
#	this command will need to be run again.
update-schema:
	cd search && cargo sqlx prepare -- --lib 

run:
	cd search && cargo run 
run-release:
	cd search && cargo run --release
flame-run:
	cd search && cargo flamegraph --dev

flame-run-release:
	cd search && cargo flamegraph 
build:
	cd search && cargo build --release 

test:
	cd search && cargo test --workspace -- --test-threads=1 ${ARGS}

docs:
	cd search && cargo doc --open --no-deps


benchmark_baseline:
	cd search && cargo bench --bench benchmarks -- --verbose

benchmark_new_baseline:
	cd search && cargo bench --bench benchmarks -- --save-baseline master

benchmark_baseline_mem_monitor:
	cd search && heaptrack cargo bench --bench benchmarks -- --verbose