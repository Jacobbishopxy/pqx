# PQX

PQX stands for Priority Queue Execution. PQX uses RabbitMQ as the message system, and serves as a subscriber for receiving messages and execute related commands.

## Structure

```txt
.
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
│       ├── test_cmd.rs
│       ├── test_cmd_subscriber.rs
│       └── test_mq.rs
├── scripts
│   └── print_csv_in_line.py
├── LICENSE
├── Makefile
└── README.md
```

## Quick startup

1. Make sure RabbitMq has been started, simply by executing `make rbmq-start`. Check [docker-compose](./docker/rabbitmq/docker-compose.yml) for composing detail.

1. Add user: `make rbmq-adduser`

1. Declare DLX (dead letter exchange): `make rbmq-setdlx`

## Todo

- [Routing & Topics](https://www.rabbitmq.com/tutorials/tutorial-four-python.html)
