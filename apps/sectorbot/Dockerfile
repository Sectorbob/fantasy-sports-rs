FROM scratch
# `make buildd` copies the statically linked binary here before building
ADD ./sectorbot.bin /sectorbot
# Secrets are NOT baked into the image. Supply them at runtime:
#   docker run --env-file .env sectorbot:latest-amd64
CMD ["/sectorbot"]
