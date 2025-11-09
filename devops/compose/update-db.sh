#!/bin/sh

docker compose -f devops/compose/compose.yaml kill db
docker compose -f devops/compose/compose.yaml down db
docker compose -f devops/compose/compose.yaml up -d db
