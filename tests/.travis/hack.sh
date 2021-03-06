#!/bin/bash

set -eux

# This file contains hacks needed to setup the travis environment correctly

if [[ "${TRAVIS_OS_NAME}" == "linux" ]]; then
  # setup cgroupsv2
  sudo mount -o remount,rw /sys/fs/cgroup
  if ! [[ -d "/sys/fs/cgroup/unified" ]]; then
    sudo mkdir -p /sys/fs/cgroup/unified
    sudo mount -t cgroup2 none /sys/fs/cgroup/unified
  fi
else
  # macos
  # upstream travis doesn't update brew and brew refuses to install packages if
  # we don't upgrade it :(
  brew update --quiet
  brew install bash fish gimme zsh
fi
