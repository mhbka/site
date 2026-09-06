# Infrastructure

This configuration creates a DigitalOcean Ubuntu 24.04 Droplet, its firewall,
and Cloudflare A records for the zone apex and `api` subdomain. The GitHub Actions workflow installs
Docker and starts the Compose application and Caddy.

## DigitalOcean access and networking

Create a DigitalOcean personal access token with read/write scope and store it
as `DIGITALOCEAN_TOKEN`. Terraform creates the Droplet and its firewall; no VPC,
subnet, or gateway resources are needed for this public deployment.

The default location is Singapore (`sgp1`) and the image is Ubuntu 24.04. The
Droplet firewall and UFW both allow TCP 22, 80, and 443. Set
`DIGITALOCEAN_SSH_USER` to `root`, which is the default user for the DigitalOcean
Ubuntu image. Narrow SSH to a fixed source range after establishing a suitable
access path.

## Migrating the existing state

This is a provider change, not an in-place instance replacement. The existing
OCI resources in the remote Terraform state must be destroyed with the old OCI
configuration and credentials before the first DigitalOcean workflow run; do
not remove them from state while they still exist. Back up the state first.
From a checkout of the prior OCI configuration, target only the OCI resources
for destruction so the existing Cloudflare records remain in state:

```sh
terraform destroy \
  -target=oci_core_instance.site \
  -target=oci_core_subnet.public \
  -target=oci_core_security_list.public \
  -target=oci_core_route_table.public \
  -target=oci_core_internet_gateway.site \
  -target=oci_core_vcn.site
```

The first DigitalOcean apply creates the Droplet, updates both Cloudflare A
records, then deploys the application.

## Remote state

The state is stored in an existing S3-compatible bucket, including Cloudflare
R2. Create the bucket before first use. Terraform must initialize its backend
before it can create resources, so this configuration cannot create its own
state bucket.

```sh
terraform init \
  -backend-config="bucket=$TF_STATE_BUCKET" \
  -backend-config="key=$TF_STATE_KEY"
```

For R2, use `auto` for `AWS_REGION` and
`https://<ACCOUNT_ID>.r2.cloudflarestorage.com` for `AWS_ENDPOINT_URL_S3`.

## GitHub Actions configuration

`.github/workflows/infra.yml` plans on pull requests and applies plus deploys
on pushes to `main`.

Create these GitHub Actions variables:

```text
DIGITALOCEAN_INSTANCE_NAME         # e.g. site
DIGITALOCEAN_REGION                # sgp1
DIGITALOCEAN_DROPLET_SIZE          # s-2vcpu-4gb
DIGITALOCEAN_IMAGE                 # ubuntu-24-04-x64
CLOUDFLARE_ZONE_ID
S3_REGION                          # auto for Cloudflare R2
S3_ENDPOINT                        # R2 account endpoint
TF_STATE_BUCKET
TF_STATE_KEY
DIGITALOCEAN_SSH_USER              # root
API_SUBDOMAIN                      # api
SITE_DOMAIN
API_DOMAIN
ACME_EMAIL
```

Create these GitHub Actions secrets:

```text
DIGITALOCEAN_TOKEN                 # DigitalOcean personal access token with read/write scope
CLOUDFLARE_API_TOKEN
S3_ACCESS_KEY_ID
S3_SECRET_ACCESS_KEY
INSTANCE_SSH_PUBLIC_KEY
INSTANCE_SSH_PRIVATE_KEY
BLOG_ENV
BACKEND_ENV
```

`INSTANCE_SSH_PUBLIC_KEY` and `INSTANCE_SSH_PRIVATE_KEY` must be the matching pair used
to access the Ubuntu instance. `BLOG_ENV` and `BACKEND_ENV` are complete `.env`
file contents. The workflow excludes `.env` files from the repository sync,
writes those files on the instance, then runs `docker compose up -d --build`.

## Caddy and Cloudflare

Set these deployment variables for Compose/Caddy, not Terraform:

```dotenv
SITE_DOMAIN=example.com
API_DOMAIN=api.example.com
ACME_EMAIL=ops@example.com
```

Caddy is the only service publishing ports 80 and 443. It proxies the apex
domain to `frontend:4321` and the API domain to `backend:8080`. Set Cloudflare
SSL/TLS mode to **Full (strict)**.
