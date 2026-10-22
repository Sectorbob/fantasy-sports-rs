FROM scratch
ADD ./target/x86_64-unknown-linux-musl/release/sectorbot /sectorbot
# Secrets are NOT baked into the image. Supply them at runtime:
#   docker run --env-file .env sectorbot:latest-amd64
CMD ["/sectorbot"]
