# WikiSearch

## Requirements
- docker
- sqlx-cli (cargo install sqlx-cli)

## Starting search API server in docker container
- in the `infrastructure` repository run `make up COMPOSE_FILE:=docker-compose-db-only.yml` -> `make migrate` -> `make add-test-data` to populate the test database 
- run `make update-schema` 
    - Install sqlx-cli (`cargo install sqlx-cli`) if it is not installed already
    - After there has been a change to the database schema or queries, this command will need to be run again.
- build docker image:
    - `make build_img`, defaults tag to: wiki_search_api:0.1
- run the image
    - `$ docker run -p 8000:8000 --rm -a stdin -a stdout <image name>:<image version>`
    - or `make run_img` for default tags
    - `--rm` makes sure image closes after you quit the shell
    - `-p` binds the port 8000 in the container to port 8000 on your machine (localhost)
    - `-a` options bind the console output and input to your shell for debugging
- The API server should now be available under `localhost:8000`

## Compiling and running locally without docker
- Install rust 1.58.1: https://www.rust-lang.org/learn/get-started
    - running `cargo build` should automatically install this version
- Install docker: https://docs.docker.com/
- `make build` or `make run`

- `./target/release/search`

## Running tests
- Tests are found in `src/tests`
- To run simply use: `make test`

## Accessing & building documentation
- `make docs` (opens in browser)

## Environment Variables
- `SEARCH_PORT`: sets the port at which search API listens (default 8000)
- `SEARCH_IP`: setst the ip address to which the search API binds (default 127.0.0.1) 
- `SQLX_OFFLINE` : if true reads sqlx-data.json at compile time to verify queries
- `DATABASE_URL` : the sql connection string 
- `RUST_LOG` : level of logging
