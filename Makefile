.PHONY: all
all: rust go

.PHONY: rust
rust:
	cargo install uniffi-bindgen-go --git https://github.com/NordSecurity/uniffi-bindgen-go --tag v0.3.0+v0.28.3
	cargo build --release

.PHONY: go
go:
	mkdir -p dist
	uniffi-bindgen-go --out-dir pkg/iroh_streamplace/generated --library ./target/release/libiroh_streamplace.dylib

.PHONY: test
test:
	go test ./pkg/...
