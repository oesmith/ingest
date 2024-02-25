#!/usr/bin/env bash

set -ex

while getopts ":fn" opt; do
  case $opt in
    f)
      fetch=1
      ;;
  esac
done

if [ -n "$fetch" ] ; then

  rm -rf tmp

  mkdir -p tmp/csv

  script/urls > tmp/urls.txt

  urls=`cat tmp/urls.txt`
  for url in $urls ; do
    filename=`basename "$url"`
    curl -L -o "tmp/csv/$filename" "$url"
    script/fixup -i "tmp/csv/$filename"
  done

fi

script/check tmp/csv/*

cargo run -r

sqlite3 howmanyleft.sqlite3 'select count(1) from models'
sqlite3 howmanyleft.sqlite3 'select count(1) from generic_models'
sqlite3 howmanyleft.sqlite3 'select count(1) from makes'
