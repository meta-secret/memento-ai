#!/usr/bin/env bash

curl -X POST \
    'http://qdrant.default.svc.cluster.local:6333/collections/probio_collection/snapshots/upload?priority=snapshot' \
    -H 'Content-Type:multipart/form-data' \
    -F 'snapshot=@/qdrant-backup/25-06-2024/probio_collection-3563913780311279-2024-06-24-23-21-27.snapshot'
