#!/bin/sh

cat <<EOF > ~/.cqlshrc
[authentication]
username = $(cat "$SCYLLA_USER_FILE")
password = $(cat "$SCYLLA_PASSWORD_FILE")

[connection]
hostname = upchime-db
port = 9042
EOF

until cqlsh upchime-db 9042 -e "DESCRIBE KEYSPACES"; do
    echo "waiting for db to open"
    sleep 5
done

cqlsh upchime-db 9042 -f ~/schema.cql
