FROM rust:1.62.1-bullseye as build
COPY . .
RUN cargo build --release --verbose

FROM gcr.io/distroless/cc-debian11
# FROM gcr.io/distroless/cc
COPY --from=build /target/release/identicon-server /

# FROM rust:1.62.1-slim-bullseye
# COPY --from=build ./target/release/identicon-server .

EXPOSE 8080

CMD ["./identicon-server"]
