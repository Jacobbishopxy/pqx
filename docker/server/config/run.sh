#!/usr/bin/env bash
# author:	Jacob Xie
# @date:	2023/07/04 15:08:42 Tuesday
# @brief:

SCRIPT_DIR=`dirname "${BASH_SOURCE-$0}"`
SCRIPT_DIR=`cd "$SCRIPT_DIR"; pwd`
DATE=`date +'%Y/%m/%d %H:%M:%S'`

start_sub() {
  echo "starting pqx subscriber $DATE"
  subscriber -q $1 &
  echo $! > subscriber.pid
  echo "done"
}


start() {
  case "$1" in
    sub)
      start_sub $2
      ;;
    all)
      start_sub $2
      ;;
  esac
  echo "all done"
}

stop_sub() {
  echo "stoping pqx subscriber"
  SUB_PID=`cat subscriber.pid`
  if [ -n "$SUB_PID" ]
  then
    `echo $SUB_PID | xargs kill `
  fi
  `rm -rf subscriber.pid`
  echo "done"
}


stop() {
  case "$1" in
    sub)
      stop_sub
      ;;
    all)
      stop_sub
      ;;
  esac
  echo "all done"
}

case "$1" in
  sub|all)
    case "$2" in
      start)
        start $1 $3
        ;;
      stop)
        stop $1
        ;;
      restart)
        stop $1
        sleep 2
        start $1 $3
        ;;
      *)
        echo "Usage: $0 {sub} {start|stop|restart}"
    esac
    ;;
  *)
    echo "Usage: $0 {sub} {start|stop|restart}"
esac
