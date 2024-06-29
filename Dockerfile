FROM rust:1.49 as builder
COPY . .
RUN cargo build --release
FROM debian:buster-slim
COPY --from=builder /target/release/rust-chat-appp /app/
CMD ["/app/rust-chat-app"]
