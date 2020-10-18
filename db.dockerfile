FROM postgres

COPY ./db_schema /docker-entrypoint-initdb.d