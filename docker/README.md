# Docker

## Containers

- Facilities

  - RabbitMQ

  - PostgreSQL

- Server

  - Pqx

## Deploy

Make sure `prod.env` is under the right location (same direction as `default.env`).

## Misc

```sh
docker run --name tmp --rm --entrypoint="" -it pqx-app:0.1 bash
```

- add `ps` command for debian-slim

```sh
apt-get update && apt-get install procps
```
