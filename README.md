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

## Todo

- [Routing & Topics](https://www.rabbitmq.com/tutorials/tutorial-four-python.html)
