# Static (musl) build and packaging for sectorbot. Run from apps/sectorbot.
#
#   make musl                              build target/x86_64-unknown-linux-musl/release/sectorbot
#   make buildd                            build the docker image (no secrets inside; use --env-file)
#   make publish BUCKET_NAME=my-bucket     upload the binary to a GCS bucket
#
# Needs: rustup target add x86_64-unknown-linux-musl, and a musl C compiler for the TLS stack
#        (Debian/Ubuntu: sudo apt install musl-tools).
SVC_NAME := sectorbot
MUSL_TAG := latest-amd64
BIN := ../../target/x86_64-unknown-linux-musl/release/$(SVC_NAME)
BUCKET_NAME ?=
export CC_x86_64_unknown_linux_musl ?= musl-gcc

musl:
	cargo build --release --target x86_64-unknown-linux-musl -p sectorbot
buildd: musl
	cp $(BIN) ./sectorbot.bin
	docker build -t $(SVC_NAME):$(MUSL_TAG) . ; status=$$?; rm -f ./sectorbot.bin; exit $$status
publishd: buildd check-bucket
	docker save -o $(SVC_NAME).tar $(SVC_NAME):$(MUSL_TAG) && \
	gsutil cp $(SVC_NAME).tar gs://$(BUCKET_NAME)/artifacts/ && \
	rm $(SVC_NAME).tar
publish: musl check-bucket
	gsutil cp $(BIN) gs://$(BUCKET_NAME)/artifacts/

check-bucket:
	@test -n "$(BUCKET_NAME)" || (echo "BUCKET_NAME is required" >&2; exit 1)

.PHONY: musl buildd publishd publish check-bucket
