docker run -d --rm --name db_integration_test -e POSTGRES_USER=psql -e POSTGRES_PASSWORD=test -e POSTGRES_DB=gehma -p 10000:5432 postgres
