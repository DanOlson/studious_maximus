#!/bin/bash

env | awk -F= '{print "export", $1"=\""substr($0, index($0,$2))"\""}' > /env.sh
cron -f
