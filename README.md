# Deployment

The frontend and backend are built as separate production images. The backend
uses the existing external Postgres/Supabase database and S3-compatible storage;
those services are not included in this Compose file.

## Configuration

Set the frontend's public values in `blog/.env`. `BACKEND_URL` must be the
public API URL that a visitor's browser can reach. These values are embedded
into the frontend image during `docker compose build`.

Configure `backend/.env` from `backend/.env.example` with the database,
Supabase JWK set, and storage credentials. `PORT` is optional and defaults to
`8080`.

## Run

```sh
docker compose up --build -d
```

The frontend listens on port `4321`; the API listens on port `8080`. For a
public deployment, place a TLS reverse proxy in front of them and route the
public API hostname to port `8080`.

After changing a frontend environment value, rebuild the frontend image:

```sh
docker compose build frontend
docker compose up -d frontend
```

Docker Desktop (or another Docker engine) must be running before building or
starting the stack.
