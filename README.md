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
│   ├── src
│   │   ├── ec
│   │   │   ├── cmd.rs
│   │   │   ├── exec.rs
│   │   │   ├── mod.rs
│   │   │   └── util.rs
│   │   ├── mq
│   │   │   ├── client.rs
│   │   │   ├── consumer.rs
│   │   │   ├── mod.rs
│   │   │   ├── publish.rs
│   │   │   └── subscribe.rs
│   │   ├── cfg.rs
│   │   ├── error.rs
│   │   └── lib.rs
│   └── tests
│       ├── test_callback.rs
│       ├── test_cmd.rs
│       ├── test_consumer.rs
│       ├── test_dlx.rs
│       ├── test_headers.rs
│       ├── test_mq.rs
│       ├── test_subscriber.rs
│       └── test_topics.rs
├── pqx-app
│   └── src
│       ├── bin
│       │   └── subscriber.rs
│       └── lib.rs
├── pqx-util
│   └── src
│       ├── cfg.rs
│       ├── error.rs
│       └── lib.rs
├── scripts
│   ├── print_csv_in_line.py
│   └── test_consumer_pub.py
├── LICENSE
├── Makefile
├── Makefile.env
└── README.md
```

## Quick startup

1. Make sure RabbitMq has been started, simply by executing `make rbmq-start`. Check [docker-compose](./docker/rabbitmq/docker-compose.yml) for composing detail.

1. Add user: `make rbmq-adduser`

1. Declare DLX (dead letter exchange): `make rbmq-setdlx`

## Test cases

- [cmd](./pqx/tests/test_cmd.rs): `cmd` module, commands composation and execution

- [mq](./pqx/tests/test_mq.rs): `mq` module, basic pub/sub

- [subscriber](./pqx/tests/test_subscriber.rs): pub/sub combined with a command executor

- [dlx](./pqx/tests/test_dlx.rs): dead letter exchange

- [topics](./pqx/tests/test_topics.rs): topic exchange

- [headers](./pqx/tests/test_headers.rs): header exchange

- [custom consumer](./pqx/tests/test_consumer.rs): a further test case from [subscriber](./pqx/tests/test_subscriber.rs), with custom consumer, command execution and logging. Moreover, a [Python script](./scripts/test_consumer_pub.py) for message publishing is also provided.
