#!/bin/sh

docker compose -f devops/compose/compose.yaml kill
docker compose -f devops/compose/compose.yaml down
