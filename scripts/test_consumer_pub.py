# @file:	test_consumer_pub.py
# @author:	Jacob Xie
# @date:	2023/06/06 22:53:12 Tuesday
# @brief:	test case of `/pqx/tests/test_consumer.rs`'s `mq_publish_success`, written in Python

# core lib
import json
import pika

# other deps
import pytz
import yaml
from pathlib import Path
import datetime as dt

# ================================================================================================
# read config
# ================================================================================================

path = Path(__file__).parent / "../pqx/conn.yml"
f = yaml.safe_load_all(path.open())
cfg = next(f)

host = cfg["host"]
port = cfg["port"]
user = cfg["user"]
pswd = cfg["pass"]
vhost = cfg["vhost"]

exchange = "rbmq-rs-exchange"
rout = "rbmq-rs-rout"

conda_env = "py310"

# ================================================================================================
# connection
# ================================================================================================


credentials = pika.PlainCredentials(username=user, password=pswd)
connection = pika.BlockingConnection(
    pika.ConnectionParameters(
        host=host, port=port, virtual_host=vhost, credentials=credentials
    )
)
channel = connection.channel()

# ================================================================================================
# publish msg
# ================================================================================================

# message construction
msg = json.dumps(
    {
        "cmd": {
            "CondaPython": {
                "dir": str(Path(__file__).parent.resolve()),
                "env": conda_env,
                "script": "print_csv_in_line.py",
            }
        },
        "time": dt.datetime.now(tz=pytz.timezone("Asia/Shanghai")).isoformat(),
    }
)
print(msg)

# publish message
channel.basic_publish(
    exchange=exchange,
    routing_key=rout,
    body=msg,
)
print("msg has been published to RabbitMQ")

connection.close()
