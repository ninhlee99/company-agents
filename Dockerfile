# syntax=docker/dockerfile:1

FROM rust:1-alpine AS build
WORKDIR /src
COPY Cargo.toml ./
COPY apps ./apps
COPY crates ./crates
COPY agents ./agents
RUN cargo build --release -p company-os -p media-worker -p llm-web-relay

FROM alpine:3.20 AS company-runtime
RUN apk add --no-cache ca-certificates && adduser -D -H -u 10001 app
USER app
WORKDIR /app
COPY --from=build /src/target/release/company-os /app/company-os
EXPOSE 8080
ENTRYPOINT ["/app/company-os"]

FROM alpine:3.20 AS media-runtime
RUN apk add --no-cache ffmpeg ca-certificates && adduser -D -H -u 10002 media
USER media
WORKDIR /app
COPY --from=build /src/target/release/media-worker /app/media-worker
ENTRYPOINT ["/app/media-worker"]

FROM alpine:3.20 AS llm-web-relay-runtime
RUN apk add --no-cache ca-certificates && adduser -D -H -u 10003 relay
USER relay
WORKDIR /app
COPY --from=build /src/target/release/llm-web-relay /app/llm-web-relay
EXPOSE 9010
ENTRYPOINT ["/app/llm-web-relay"]
