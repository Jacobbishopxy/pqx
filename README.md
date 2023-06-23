# PQX

PQX stands for Priority Queue Execution. PQX uses RabbitMQ as the message system, and serves as a subscriber for receiving messages and execute related commands.

PS:

1. Retry functionality is based on RabbitMQ plugin `delayed_message_exchange`, check [this](https://github.com/rabbitmq/rabbitmq-delayed-message-exchange) for more detail;

## Project Structure

- `pqx-util`: utilities

  - `cfg`: config and misc

  - `db`: persistent connection

  - `logging`: logging utils

  - `mq`: RabbitMQ management APIs

- `pqx`: library

  - `ec`: commands and executors

  - `mq`: publisher and subscriber

- `pqx-app`: applications

  - `initiator`: check existences | create tables | declare exchanges, queues and etc.

  - `subscriber`: app

```txt
.
├── pqx
│   └── src
│       ├── ec
│       │   ├── cmd.rs
│       │   ├── exec.rs
│       │   ├── mod.rs
│       │   └── util.rs
│       ├── mq
│       │   ├── client.rs
│       │   ├── consumer.rs
│       │   ├── mod.rs
│       │   ├── predefined.rs
│       │   ├── publish.rs
│       │   └── subscribe.rs
│       ├── error.rs
│       └── lib.rs
├── pqx-app
│   └── src
│       ├── bin
│       │   ├── initiator.rs
│       │   └── subscriber.rs
│       ├── entities
│       │   ├── message_history.rs
│       │   ├── message_result.rs
│       │   └── mod.rs
│       ├── adt.rs
│       ├── cfg.rs
│       ├── execution.rs
│       ├── lib.rs
│       └── persistence.rs
├── pqx-util
│   └── src
│       ├── cfg.rs
│       ├── db.rs
│       ├── error.rs
│       ├── lib.rs
│       ├── logging.rs
│       └── mq.rs
└── LICENSE
```

## Message flow (Pqx-app)

![app](./app.svg)

## Quick startup

### Tests

1. Build image for RabbitMQ (including plugins): `make facilities-build`

1. Make sure RabbitMQ and PostgreSQL has been started, simply by executing `make facilities-start`. Check [docker-compose](./docker/facilities/docker-compose.yml) for composing detail.

1. Add RabbitMQ user: `make mq-adduser`; for supervisor role (enable website operation): `make mq-supervisor`

1. Running the test cases

### Deploy

1. Following the same steps described in [Tests](#tests)

1. Build image for Pqx: `make pqx-build`

1. Follow the template files under `./docker/server/config`, create config files: `conn.yml` & `init.yml`.

1. Run a Pqx container: `make pqx-setup`

## Test cases

- [cmd](./pqx/tests/test_cmd.rs): `cmd` module, commands composition and execution

- [mq](./pqx/tests/test_mq.rs): `mq` module, basic pub/sub

- [subscriber](./pqx/tests/test_subscriber.rs): pub/sub combined with a command executor

- [dlx](./pqx/tests/test_dlx.rs): dead letter exchange

- [topics](./pqx/tests/test_topics.rs): topic exchange

- [headers](./pqx/tests/test_headers.rs): header exchange

- [custom consumer](./pqx/tests/test_consumer.rs): a further test case from [subscriber](./pqx/tests/test_subscriber.rs), with custom consumer, command execution and logging. Moreover, a [Python script](./scripts/test_consumer_pub.py) for message publishing is also provided.

- [callback registration](./pqx/tests/test_callback.rs): connection & channel callback registration

- [delay retry](./pqx/tests/test_retry.rs): based on plugin [delayed_message_exchange](https://github.com/rabbitmq/rabbitmq-delayed-message-exchange), implementation of message retry

- [message persistence](./pqx-app/tests/test_persistence.rs): database interaction

- [mq api](./pqx-util/tests/test_mq.rs): RabbitMQ management APIs
