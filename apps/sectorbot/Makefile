# set BUCKET_NAME to the GCS bucket to publish to, e.g. `make publish BUCKET_NAME=my-bucket`
BUCKET_NAME ?=
SVC_NAME := sectorbot
MUSL_TAG := latest-amd64

musl:
	cargo build --release --target x86_64-unknown-linux-musl
buildd: musl
	docker build -t $(SVC_NAME):$(MUSL_TAG) .
publishd: buildd check-bucket
	docker save -o $(SVC_NAME).tar $(SVC_NAME):$(MUSL_TAG) && \
	gsutil cp $(SVC_NAME).tar gs://$(BUCKET_NAME)/artifacts/ && \
	rm $(SVC_NAME).tar
publish: musl check-bucket
	gsutil cp ./target/x86_64-unknown-linux-musl/release/$(SVC_NAME) gs://$(BUCKET_NAME)/artifacts/

check-bucket:
	@test -n "$(BUCKET_NAME)" || (echo "BUCKET_NAME is required" >&2; exit 1)

.PHONY: musl buildd publishd publish check-bucket
