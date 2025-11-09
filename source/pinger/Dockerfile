FROM alpine:3.22.2

RUN apk add gcompat --no-cache
RUN apk add libgcc --no-cache
RUN mkdir /var/log/app
RUN addgroup app
RUN adduser -D -G app app
COPY --chown=app:app target/debug/pinger /home/app/
RUN chown app:app /var/log/app
USER app

CMD ["/home/app/pinger"]
