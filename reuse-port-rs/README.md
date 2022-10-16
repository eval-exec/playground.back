# try reproduce reuse port issue

## usage `./reuse-port-rs [reuse port] [optional: remote address]`
example :
`./reuse-port-rs 8080` # run a reuse port listener on `8080`
`./reuse-port-rs 8080 [remote public ip]:18080` # run a reuse port listener on 18080, and
dial `[remote public ip]:18080`

## expected

In host 1: execute `cargo run -- 8080`
In host 2: execute `cargo run -- 18080 [remote public ip]:8080`

## expected
docker build -t reuse-port-rs:v0 .

In host 1: execute `docker run --rm --network host reuse-port-rs:v0 8080`
In host 2: execute `cargo run -- 18080 [remote public ip]:8080`

## expected
In host 1: execute `docker run --rm -p 8080:8080 reuse-port-rs:v0 8080`
In host 2: execute `cargo run -- 18080 [remote public ip]:8080`
