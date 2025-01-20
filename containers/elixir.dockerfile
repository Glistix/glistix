FROM elixir:latest

ARG TARGETARCH
COPY glistix-${TARGETARCH} /bin/glistix

CMD ["glistix"]
