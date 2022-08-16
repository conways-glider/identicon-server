FROM rust:1.63-bullseye as build
COPY . .
RUN cargo build --release --verbose
RUN ls -la /target/release/

# FROM rust:1.62.1-bullseye
# FROM gcr.io/distroless/cc-debian11
# FROM gcr.io/distroless/static-debian11
FROM gcr.io/distroless/cc
COPY --from=build /target/release/identicon-server /

EXPOSE 8080
ENV RUST_LOG info

CMD ["./identicon-server"]
