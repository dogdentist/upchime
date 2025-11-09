#!/bin/sh

echo "loading init.conf"
source ./init.conf

echo "configuring docker compose"
echo -n $DB_USERNAME > devops/compose/secrets/db_username
echo -n $DB_PASSWORD > devops/compose/secrets/db_password

echo "LOG_RETENTION='$LOG_RETENTION'" > devops/compose/.env
echo "PINGER_DB_SYNC_INTERVAL='$PINGER_DB_SYNC_INTERVAL'" >> devops/compose/.env
