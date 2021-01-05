include ../ENV.list
export $(shell sed 's/=.*//' ../ENV.list) 

.PHONY: release, test, dev, run

release:
	cargo update
	cargo test
	cargo build --release
	strip target/release/customer_microservice

run:
	cargo run

build:
	cargo update
	cargo build
	cargo test

dev:
	# . ./ENV.sh; backper
	cargo run;

test:
	cargo test