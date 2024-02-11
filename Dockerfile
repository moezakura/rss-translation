FROM rust:1.76-bookworm as builder

WORKDIR /app
ADD . /app

RUN cargo build --release


FROM gcr.io/distroless/cc-debian12

COPY --from=builder /app/target/release/rss-trans /app

ENV GOOGLE_APPLICATION_CREDENTIALS=/app/service-account.json
ENV GOOGLE_CLOUD_PROJECT=your-project-id

ENV CACHE_MODE=

ENV WEB_DAV_URL=
ENV WEB_DAV_USER_ID=
ENV WEB_DAV_USER_PASSWORD=

ENV S3_ENDPOINT_URL=
ENV S3_BUCKET_NAME=
ENV S3_REGION=
ENV AWS_ACCESS_KEY_ID=
ENV AWS_SECRET_ACCESS_KEY=

EXPOSE 8080

CMD ["/app"]
