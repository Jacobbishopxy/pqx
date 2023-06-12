# PQX

PQX stands for Priority Queue Execution. PQX uses RabbitMQ as the message system, and serves as a subscriber for receiving messages and execute related commands.

## Structure

- `pqx-util`: util functions

- `pqx`

  - `ec`: commands and executors

  - `mq`: publisher and subscriber

- `mq-api`: RabbitMQ management API

- `pqx-app`: applications

```txt
.
├── mq-api
│   └── src
│       ├── client.rs
│       ├── error.rs
│       ├── lib.rs
│       └── query.rs
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
│       │   ├── publish.rs
│       │   └── subscribe.rs
│       ├── cfg.rs
│       ├── error.rs
│       └── lib.rs
├── pqx-app
│   └── src
│       ├── bin
│       │   ├── initiator.rs
│       │   └── subscriber.rs
│       └── lib.rs
├── pqx-util
│   └── src
│       ├── cfg.rs
│       ├── error.rs
│       └── lib.rs
├── LICENSE
├── Makefile
├── Makefile.env
└── README.md
```

![app](./app.svg)

## Quick startup

1. Build image for RabbitMQ: `make facilities-build`

1. Make sure RabbitMQ and PostgreSQL has been started, simply by executing `make facilities-start`. Check [docker-compose](./docker/facilities/docker-compose.yml) for composing detail.

1. Add RabbitMQ user: `make mq-adduser`; for supervisor role (enable website operation): `make mq-supervisor`

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
