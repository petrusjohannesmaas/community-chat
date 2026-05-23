FROM rust:1.86 AS builder
WORKDIR /app
COPY . .
RUN cargo build --release

FROM debian:bookworm-slim
WORKDIR /app
COPY --from=builder /app/target/release/chat-server .
ENV DATABASE_URL=sqlite:/data/chat.db
EXPOSE 3030
CMD ["./chat-server"]
