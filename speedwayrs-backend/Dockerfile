FROM rust:latest

ENV DATABASE_URL="postgres://srs_backend:srs-dev@localhost:6000/speedwayrs"

WORKDIR /backend
COPY . .
RUN cargo build --release

CMD ["cargo", "run", "--release"]
