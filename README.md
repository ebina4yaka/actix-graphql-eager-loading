# actix-graphql-eager-loading
## development
run
```bash
docker-compose up -d
```
```bash
diesel migration run
```
add .env example
```.env
DATABASE_URL=postgres://postgres:postgres@localhost/graphql_eager_loading_sample
RUST_LOG=debug,actix_web=debug
```
