#!/bin/sh

docker build -t rustc5 . || exit 1
docker run rustc5 true || exit 1
docker cp $(docker ps -aq | head -n1):/lovely_touching/target/release/lovely_touching .
