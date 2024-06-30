#!/usr/bin/env bash

mkdir build
kcl run > build/Taskfile.yml

task -t build $1