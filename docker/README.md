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

- temporary access image for debugging

```sh
docker run --name tmp --rm --entrypoint="" -it pqx-app:0.1 bash
```
