FROM docker.io/library/rust:alpine AS builder

## Use a temp project to seed the dependencies
RUN USER=root cargo new --bin actix-geo-widget
WORKDIR /actix-geo-widget
COPY ./Cargo.toml ./Cargo.toml
RUN cargo build --release

## Clean up the temp files
RUN rm src/*.rs
RUN rm ./target/release/deps/actix-geo-widget*

## Copy the source files for the project
COPY ./src ./src

## Build the release
RUN cargo build --release

## Use a smaller base image for a smaller final image
FROM docker.io/library/alpine:latest AS final

## Copy the binary from the builder 
COPY --from=builder /actix-geo-widget/target/release/actix-geo-widget /actix-geo-widget

# Add a user account
# No need to run as root in the container
RUN addgroup -S appgroup \
    && adduser -S appuser -G appgroup

# Run all future commands as appuser
USER appuser

# podman build --build-arg MAXMIND_API_KEY=${MAXMIND_API_KEY}
ARG MAXMIND_API_KEY=NOT-SET
COPY ./get_maxmind_database.sh ./get_maxmind_database.sh
RUN ./get_maxmind_database.sh -u -e GeoLite2-ASN,GeoLite2-City -k ${MAXMIND_API_KEY}

## Setup the entrypoint with the binary
ENTRYPOINT ["/bin/sh", "-c", "/actix-geo-widget"]