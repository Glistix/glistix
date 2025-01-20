FROM scratch

ARG TARGETARCH
COPY glistix-${TARGETARCH} /bin/glistix

CMD ["glistix"]
