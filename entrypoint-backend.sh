#!/bin/sh

if [ -z "${PG_HOST}" ]; then
  /usr/local/bin/backend --work-dir /workdir
else
  /usr/local/bin/backend --work-dir /workdir --pg-user "${PG_USER}" --pg-password "${PG_PASSWORD}" --pg-database "${PG_DATABASE}" --pg-url="${PG_HOST}"
fi