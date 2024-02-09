FROM rust:1.76-bookworm as builder

WORKDIR /app
ADD . /app

RUN cargo build --release


FROM gcr.io/distroless/cc-debian12

COPY --from=builder /app/target/release/rss-trans /app

ENV GOOGLE_APPLICATION_CREDENTIALS=/app/service-account.json
ENV GOOGLE_CLOUD_PROJECT=your-project-id

EXPOSE 8080

CMD ["/app"]
