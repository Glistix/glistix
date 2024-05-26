FROM erlang:alpine

ARG TARGETARCH
COPY glistix-${TARGETARCH} /bin/glistix

CMD ["glistix"]
