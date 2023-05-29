# @file:	simple_send.py
# @author:	Jacob Xie
# @date:	2023/05/29 23:33:22 Monday
# @brief:

import pika

connection = pika.BlockingConnection(pika.ConnectionParameters(host="localhost"))
channel = connection.channel()

channel.queue_declare(queue="hello")

channel.basic_publish(exchange="", routing_key="hello", body="Hello World!")
print(" [x] Sent 'Hello World!'")

connection.close()
