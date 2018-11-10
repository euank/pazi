#!/bin/bash

set -eux

# This file contains hacks needed to setup the travis environment correctly

if [[ "${TRAVIS_OS_NAME}" == "linux" ]]; then
  # setup cgroupsv2
  mount -o remount,rw /sys/fs/cgroup
  if ! [[ -d "/sys/fs/cgroup/unified" ]]; then
    mkdir -p /sys/fs/cgroup/unified
    mount -t cgroup2 none /sys/fs/cgroup/unified
  fi
fi
