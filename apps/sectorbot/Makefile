BUCKET_NAME := host-c8ac28b0.example.invalid
SVC_NAME := sectorbot
MUSL_TAG := latest-amd64

musl:
	cargo build --release --target x86_64-unknown-linux-musl
buildd: musl
	docker build -t $(SVC_NAME):$(MUSL_TAG) .
publishd: buildd
	docker save -o $(SVC_NAME).tar $(SVC_NAME):$(MUSL_TAG) && \
	gsutil cp $(SVC_NAME).tar gs://$(BUCKET_NAME)/artifacts/ && \
	rm $(SVC_NAME).tar
publish: musl
	gsutil cp ./target/x86_64-unknown-linux-musl/release/$(SVC_NAME) gs://$(BUCKET_NAME)/artifacts/
