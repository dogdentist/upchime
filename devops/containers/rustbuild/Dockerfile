FROM alpine:3.22.2

ARG UID
ARG GID

RUN apk add bash --no-cache
RUN apk add rust --no-cache
RUN apk add cargo --no-cache
RUN apk add rustfmt --no-cache
RUN apk add rust-src --no-cache
RUN apk add yaml-dev --no-cache
RUN apk add openssl-dev --no-cache

RUN if [ "$UID" != "0" ]; then \
        addgroup -g ${GID} developer && \
        adduser -u ${UID} -D -G developer developer; \
    fi

RUN if [ "$UID" != "0" ]; then \
        USER developer; \
    fi

ENV PATH=/usr/bin/:$PATH
RUN mkdir -p /home/developer/project

RUN if [ "$UID" != "0" ]; then \
        chown developer:developer /home/developer/project; \
    fi

WORKDIR /home/developer/project
