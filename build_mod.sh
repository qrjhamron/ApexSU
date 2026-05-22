#!/bin/sh
make -C /lib/modules/$(uname -r)/build M=/root/ApexSU/kernel modules
