FROM docker.io/library/rust:alpine AS builder

## Add os build dependencies
RUN apk add --no-cache musl-dev=1.2.5-r0

## Copy the source files for the project
WORKDIR /actix-geo-widget
COPY ./Cargo.toml ./Cargo.toml
COPY ./src ./src

## Build the release
RUN cargo build --release

## Use a smaller base image for a smaller final image
FROM docker.io/library/alpine:3 AS final

## Copy the binary from the builder 
COPY --from=builder /actix-geo-widget/target/release/actix-geo-widget \
    /usr/local/bin/actix-geo-widget

# Add a user account
# No need to run as root in the container
RUN addgroup -S appgroup \
    && adduser -S appuser -G appgroup

# Run all future commands as appuser
USER appuser

## Setup the entrypoint with the binary
ENTRYPOINT ["/usr/local/bin/actix-geo-widget"]