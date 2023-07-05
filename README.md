# PQX

PQX stands for Priority Queue Execution. Inspired by [the official tutorial](https://www.rabbitmq.com/tutorials/tutorial-six-python.html), PQX-APP uses RabbitMQ as the message system, and serves as a RPC client which pulls messages from MQ, deserializes messages and executes commands. PQX-APP can also be placed in different machines in order to execute machine-specified commands (by `mailing_to` field, see below).

Retry functionality is based on RabbitMQ plugin `delayed_message_exchange`, check [this](https://github.com/rabbitmq/rabbitmq-delayed-message-exchange) for more detail.

Bin files provided, currently:

- [inspector](./pqx-app/src/bin/inspector.rs): inspecting database table schemas, MQ settings/status and etc.

- [initiator](./pqx-app/src/bin/initiator.rs): initializing data, such as database tables, MQ settings and etc.

- [rectifier](./pqx-app/src/bin/rectifier.rs): modifying initialized data (delete/recreate/...)

- [subscriber](./pqx-app/src/bin/subscriber.rs): consuming message from the MQ, and execute commands

- [publisher](./pqx-app/src/bin/publisher.rs): sending message to the MQ

A full command in Json expression looks like this 🧐:

```json
{
    "mailing_to": [
        {
            "unique_key": "h1"
        },
        {
            "unique_key": "h2",
            "common_key": "dev"
        }
    ],
    "config": {
        "retry": 5,
        "poke": 60,
        "waiting_timeout": 180,
        "consuming_timeout": 270
    },
    "cmd": {
        "CondaPython": {
            "env": "py310",
            "dir": "$HOME/Code/pqx/scripts",
            "script": "print_csv_in_line.py"
        }
    }
}
```

where:

- `mailing_to` a list of matching criteria (logic 'or', meaning this message will be sent multiple times), mailing to the queues' who match one of these criteria. if `mailing_to` is empty, then send to all queues (header-exchange mechanism);

- `retry` the number of retries, default `0`;

- `poke` retry frequency in *seconds*;

- `waiting_timeout` the message lives in the queue (*seconds*), default infinity;

- `consuming_timeout` the `acking` timeout in a consumer (*seconds*);

- `cmd` the command needs to be executed, for more detail see `CmdArg` in [adt.rs](./pqx/src/ec/cmd.rs).

<details>
<summary>and the full definition in Rust:</summary>

```rs
pub struct Command {
    pub mailing_to: Vec<HashMap<String, String>>,
    pub config: Config,
    pub cmd: CmdArg,
}

pub struct Config {
    pub retry: Option<u8>,
    pub poke: Option<u16>,
    pub waiting_timeout: Option<u32>,
    pub consuming_timeout: Option<u32>,
}

pub enum CmdArg {
    Ping {
        addr: String,
    },
    Bash {
        cmd: Vec<String>,
    },
    Ssh {
        ip: String,
        user: String,
        cmd: Vec<String>,
    },
    Sshpass {
        ip: String,
        user: String,
        pass: String,
        cmd: Vec<String>,
    },
    CondaPython {
        env: String,
        dir: String,
        script: String,
    },
    DockerExec {
        container: String,
        cmd: Vec<String>,
    },
}
```

</details>

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
│       │   ├── inspector.rs
│       │   ├── publisher.rs
│       │   ├── rectifier.rs
│       │   └── subscriber.rs
│       ├── entities
│       │   ├── message_history.rs
│       │   ├── message_result.rs
│       │   └── mod.rs
│       ├── adt.rs
│       ├── cfg.rs
│       ├── exec.rs
│       ├── lib.rs
│       └── persist.rs
├── pqx-util
│   └── src
│       ├── db.rs
│       ├── error.rs
│       ├── lib.rs
│       ├── log.rs
│       ├── misc.rs
│       └── mq.rs
└── LICENSE
```

## Message flow (Pqx-app)

![app](./app.svg)

## Quick startup

### Tests

1. Build image for RabbitMQ (including plugins): `make facilities-build`

1. Make sure RabbitMQ and PostgreSQL has been started, simply by executing `make facilities-setup`. Check [docker-compose](./docker/facilities/docker-compose.yml) for composing detail.

1. Add RabbitMQ user: `make mq-adduser`; for supervisor role (enable website operation): `make mq-supervisor`

1. Running the test cases

### Deploy

1. Following the same steps described in [Tests](#tests)

1. Build image for Pqx: `make pqx-build`

1. Create config files: `make init-config`, modify these configs `conn.yml` & `init.yml`.

1. Build and run a Pqx container: `make pqx-build` then `make pqx-setup`

1. Check container & initialization's availability: `docker exec pqx-dev inspector -o insp`

1. Create tables for message persistence and declare exchanges, queues and bindings: `docker exec pqx-dev initiator -o init`

1. Subscribe to a specific queue: `docker exec pqx-dev ./run.sh sub start`, make sure `./docker/server/config/secret.env` has been filled

1. Stop a subscriber: `docker exec pqx-dev ./run.sh sub stop`

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

## Known issue

- If `retry` happened, message would send back to the original exchange, and at the moment as a header-typed exchange, delayed-exchange would deliver this `retry` message to all the matched queues, which means if one consumer failed to process this message, all the other consumers would receive this message again. Hence, a strict publish header should be introduced into the message's header so that delayed-exchange could deliver the `retry` message to the right place.

- Delayed exchange cannot be removed unless used 'disable plugin' technique, see [Makefile](./Makefile) `mq-disable-delayed-exchange`, and `mq-enable-delayed-exchange`.

## Todo

- list all consumers (simply by `MqQuery`)

- flexible `publisher` (not only read task from Json file)

- enhance `Command`, for instance accepting string replacement in `CmdArg`

- Module `dynamic`: dynamically set/del exchange/queue/binding.
