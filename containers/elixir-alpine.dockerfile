FROM elixir:alpine

ARG TARGETARCH
COPY glistix-${TARGETARCH} /bin/glistix

CMD ["glistix"]
