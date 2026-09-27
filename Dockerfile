# syntax=docker/dockerfile:1

FROM rust:1-alpine AS build
WORKDIR /src
COPY Cargo.toml ./
COPY apps ./apps
COPY crates ./crates
RUN cargo build --release -p company-os

FROM alpine:3.20
RUN adduser -D -H -u 10001 app
USER app
WORKDIR /app
COPY --from=build /src/target/release/company-os /app/company-os
EXPOSE 8080
ENTRYPOINT ["/app/company-os"]
