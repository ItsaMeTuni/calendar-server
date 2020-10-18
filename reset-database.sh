#!/usr/bin/env bash

# This script removes the dbdata volume and rebuilds the db image. This is done
# so that you can easily update the database schema. The rebuild will copy everything
# from db_schema into the image and when the container boots with an empty dbdata volume
# it will execute all .sql files in db_schema.


read -p "This will delete the database volume, you'll lose all data in the db. Are you sure? [y/N]" -n 1 -r
echo    # (optional) move to a new line
if [[ $REPLY =~ ^[Yy]$ ]]
then

    # stop containers
    docker-compose down

    # remove all volumes related to the calendarold-server project (any volumes that have a name
    # starting with "calendarold-server")
    docker volume rm calendar-server_dbdata

    # build db image
    docker-compose build db

    echo Database image was reset and volume cleared. Execute \"docker-compose up\" to start a clean database.

fi