# Deployment

The frontend and backend are built as separate production images. The backend
uses the existing external Postgres/Supabase database and S3-compatible storage;
those services are not included in this Compose file.

## Configuration

Set the frontend's public values in `BLOG_ENV`. `BACKEND_URL` must be the
public API URL that a visitor's browser can reach. GitHub Actions writes these
values to `blog/.env` while building the frontend image.

Configure `backend/.env` from `backend/.env.example` with the database,
Supabase JWK set, and storage credentials. `PORT` is optional and defaults to
`8080`.

## Run

Production images are built in GitHub Actions and published to GitHub Container
Registry. The server pulls the commit-tagged images and starts them with Docker
Compose; it does not build the applications.

The frontend listens on port `4321`; the API listens on port `8080`. For a
public deployment, place a TLS reverse proxy in front of them and route the
public API hostname to port `8080`.

The `build-backend` and `build-frontend` jobs use independent GitHub Actions
Buildx caches. The backend's existing `cargo-chef` stages cache compiled Cargo
dependencies separately from application source changes.
