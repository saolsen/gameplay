FROM ubuntu:latest
LABEL authors="steve"

ENTRYPOINT ["top", "-b"]