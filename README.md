# PQX

PQX stands for Priority Queue Execution. PQX uses RabbitMQ as the message system, and serves as a subscriber for receiving messages and execute related commands.

## Structure

```txt
.
├── docker
│   ├── rabbitmq
│   │   ├── data
│   │   └── docker-compose.yml
│   ├── server
│   └── README.md
├── pqx
│   ├── src
│   │   ├── ec
│   │   │   ├── cmd.rs
│   │   │   ├── mod.rs
│   │   │   └── util.rs
│   │   ├── mq
│   │   │   ├── client.rs
│   │   │   ├── consumer.rs
│   │   │   ├── message.rs
│   │   │   ├── mod.rs
│   │   │   ├── publish.rs
│   │   │   └── subscribe.rs
│   │   ├── error.rs
│   │   └── lib.rs
│   └── tests
│       ├── test_cmd.rs
│       └── test_mq.rs
├── scripts
│   ├── print_csv_in_line.py
│   └── sample.csv
├── LICENSE
├── Makefile
└── README.md
```
