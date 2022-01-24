# Requirements
- docker

# Starting search API server in docker container
- build docker image:
    - `$ docker build . -t <image name>:<image version> (i.e. search:1.0)`
- run the image
    - `$ docker run -p 80:80 --rm -a stdin -a stdout <image name>:<image version>`
    - `--rm` makes sure image closes after you quit the shell
    - `-p` binds the port 80 in the container to port 80 on your machine (localhost)
    - `-a` options bind the console output and input to your shell for debugging
- The API server should now be available under `localhost:80`

# Compiling and running locally without docker
- Install rust 1.58.1: https://www.rust-lang.org/learn/get-started
    - running `cargo build` should automatically install this version
- Install docker: https://docs.docker.com/
- `cargo build --release`
- `./target/release/search`

# Running tests
    - Tests are found in `src/tests`
    - To run simply use: `cargo test`

# Environment Variables
- `SEARCH_PORT`: sets the port at which search API listens (default 80)
- `SEARCH_IP`: setst the ip address to which the search API binds (default 127.0.0.1, for docker) 
