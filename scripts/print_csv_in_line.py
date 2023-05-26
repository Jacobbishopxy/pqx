# @file:	print_csv_in_line.py
# @author:	Jacob Xie
# @date:	2023/05/24 22:12:20 Wednesday
# @brief:

import time
import datetime as dt
import random

with open("./sample.csv") as f:
    count = 0
    while 1:
        line = f.readline()

        if not line:
            break
        # if Rust commands not using "python -u dev.py", then flush is required
        # print(f"Line{count}: {line.strip()}", flush=True)
        print(f"{dt.datetime.now()} Line{count}: {line.strip()}")
        count += 1

        time.sleep(random.randint(1, 5))
