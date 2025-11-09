FROM scylladb/scylla-cqlsh:v6.0.27

RUN useradd -m -s /bin/bash app
RUN cp -r /root/.local /home/app/
ENV PATH=$PATH:/home/app/.local/bin
USER app
COPY --chown=app:app schema.cql /home/app/schema.cql
COPY --chown=app:app init.sh /home/app/init.sh
WORKDIR /home/app

ENTRYPOINT [ "/home/app/init.sh" ]
