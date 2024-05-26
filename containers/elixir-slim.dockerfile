FROM elixir:slim

ARG TARGETARCH
COPY glistix-${TARGETARCH} /bin/glistix

CMD ["glistix"]
