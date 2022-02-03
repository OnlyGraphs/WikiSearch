# Requirements
- docker
- sqlx-cli (cargo install sqlx-cli)

# Starting search API server in docker container
- Set the database URL: `export DATABASE_URL=postgresql://postgres:password@localhost:8001/only_graph`
- run `make up` -> `make migrate` -> `make add-test-data` to populate the test database (in the infrastructure repository)
- run `make update-schema` 
    - Install sqlx-cli (`cargo install sqlx-cli`) if it is not installed already
    - After there has been a change to the database schema or queries, this command will need to be run again.
- build docker image:
    - `make build_img`, defaults tag to: wiki_search_api:0.1
- run the image
    - `$ docker run -p 8000:8000 --rm -a stdin -a stdout <image name>:<image version>`
    - or `make run_img` for default tags
    - `--rm` makes sure image closes after you quit the shell
    - `-p` binds the port 80 in the container to port 80 on your machine (localhost)
    - `-a` options bind the console output and input to your shell for debugging
- The API server should now be available under `localhost:80`

# Compiling and running locally without docker
- Install rust 1.58.1: https://www.rust-lang.org/learn/get-started
    - running `cargo build` should automatically install this version
- Install docker: https://docs.docker.com/
- `make build` or `make run`

- `./target/release/search`

# Running tests
- Tests are found in `src/tests`
- To run simply use: `make test`

# Accessing & building documentation
- `make docs` (opens in browser)

# Environment Variables
- `SEARCH_PORT`: sets the port at which search API listens (default 80)
- `SEARCH_IP`: setst the ip address to which the search API binds (default 127.0.0.1) 
