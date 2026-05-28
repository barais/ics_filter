FROM rust:1.95-trixie AS  builder
WORKDIR /usr/src/calendar-filter
COPY . .
RUN cargo build --release


FROM debian:trixie-slim
RUN apt-get update && apt-get install -y \
    ca-certificates \
    && update-ca-certificates \
    && rm -rf /var/lib/apt/lists/*
#RUN apt-get update && apt-get install -y extra-runtime-dependencies && rm -rf /var/lib/apt/lists/*
COPY --from=builder /usr/src/calendar-filter/target/release/calendar_filter /usr/local/bin/calendar-filter
CMD ["calendar-filter"]