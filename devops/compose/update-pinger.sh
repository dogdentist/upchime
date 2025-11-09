#!/bin/sh

docker compose -f devops/compose/compose.yaml kill pinger
docker compose -f devops/compose/compose.yaml down pinger
docker compose -f devops/compose/compose.yaml up -d pinger
