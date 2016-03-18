#!/bin/bash

source /etc/pingd.conf
mkdir -p /var/lib/pingd
pingd /var/lib/pingd/pings.db $hostname $address
