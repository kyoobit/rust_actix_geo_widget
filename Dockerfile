FROM docker.io/library/rust:alpine AS builder

## Add os build dependencies
RUN apk add --no-cache musl-dev=1.2.5-r0

## Copy the source files for the project
WORKDIR /actix-geo-widget
COPY ./Cargo.toml ./Cargo.toml
COPY ./src ./src

## Build the release
RUN cargo build --release

## Use a dirstroless image to run the compiled application binary
## https://github.com/GoogleContainerTools/distroless
## https://github.com/GoogleContainerTools/distroless/blob/main/cc/README.md
## https://github.com/GoogleContainerTools/distroless/blob/main/examples/rust/Dockerfile
FROM gcr.io/distroless/cc-debian12:nonroot AS final

## Copy the compiled application binary from the builder
COPY --from=builder /actix-geo-widget/target/release/actix-geo-widget \
    /usr/local/bin/actix-geo-widget

## Setup the entrypoint with the binary
ENTRYPOINT ["/usr/local/bin/actix-geo-widget"]
## Example usage:
## podman build --tag actix-geo-widget:${TAG:=v1} .
## 
## podman run --rm --name actix-geo-widget --detach \
## --volume ./GeoLite2-ASN.mmdb:/var/db/GeoLite2-ASN.mmdb:ro \
## --volume ./GeoLite2-City.mmdb:/var/db/GeoLite2-City.mmdb:ro \
## --publish 8893:8893/tcp actix-geo-widget:v1 --verbose --port 8893 \
## --asn-database-file /var/db/GeoLite2-ASN.mmdb \
## --city-database-file /var/db/GeoLite2-City.mmdb
## 
## See available appliaction options:
## podman run --rm --name actix-geo-widget-help actix-geo-widget:${TAG:=v1} --help