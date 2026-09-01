# Infrastructure

This configuration orders an OVHcloud VPS, creates Cloudflare A records for the
zone apex and `api` subdomain, and bootstraps Docker plus a minimal UFW firewall
on the VPS. It deliberately does not contain secrets.

## Prerequisites

- The domain is already an active Cloudflare zone. Its nameservers must already
  point to Cloudflare.
- An OVH API application with the permissions needed to order and read VPSs,
  and a default payment method in OVHcloud.
- An OVH image ID compatible with the VPS plan. OVH requires an image ID when a
  public SSH key is supplied during VPS creation.
- Terraform 1.5 or newer and the matching SSH private key available locally.

Set the provider credentials in your shell, then copy and fill the example:

```sh
export OVH_ENDPOINT=ovh-eu
export OVH_APPLICATION_KEY=...
export OVH_APPLICATION_SECRET=...
export OVH_CONSUMER_KEY=...
export CLOUDFLARE_API_TOKEN=...

cp terraform.tfvars.example terraform.tfvars
terraform init \
  -backend-config="bucket=$TF_STATE_BUCKET" \
  -backend-config="key=$TF_STATE_KEY"
terraform plan
terraform apply
```

The Cloudflare token needs `DNS:Read` and `DNS:Edit` for the zone.

## Remote state

The state is stored in an existing S3-compatible bucket. For OVHcloud, create
a versioned High Performance Object Storage bucket and an S3 user with access
limited to that bucket before the first `terraform init`. In GitHub Actions,
provide its settings through the environment:

```yaml
env:
  TF_STATE_BUCKET: your-terraform-state-bucket
  TF_STATE_KEY: site/production/terraform.tfstate
  AWS_REGION: gra
  AWS_ENDPOINT_URL_S3: https://s3.gra.perf.cloud.ovh.net/
  AWS_ACCESS_KEY_ID: ${{ secrets.OVH_S3_ACCESS_KEY_ID }}
  AWS_SECRET_ACCESS_KEY: ${{ secrets.OVH_S3_SECRET_ACCESS_KEY }}
```

Then initialize Terraform in the workflow:

```yaml
- name: Initialize Terraform
  working-directory: infra
  run: >-
    terraform init -input=false
    -backend-config="bucket=$TF_STATE_BUCKET"
    -backend-config="key=$TF_STATE_KEY"
```

`AWS_REGION` and `AWS_ENDPOINT_URL_S3` are read directly by the S3 backend;
the bucket and state key are passed as partial backend configuration because
Terraform has no environment variables for those two backend arguments.

The state bucket cannot be created by this same configuration because Terraform
must initialize its backend before it can create any resources. A separate
bootstrap stack is appropriate if you want bucket creation to be automated.

## Deploying the application

The GitHub Actions deployment should copy this repository's `compose.yaml` and
`infra/Caddyfile` to the VPS, create a root `.env` with the public hostnames,
and then run `docker compose up -d --build`:

```dotenv
SITE_DOMAIN=example.com
API_DOMAIN=api.example.com
ACME_EMAIL=ops@example.com
```

These are Compose/Caddy deployment variables, not Terraform variables. They
are intentionally not included in `terraform.tfvars`.

Only Caddy exposes ports 80 and 443. The `frontend` and `backend` containers
remain private on the Compose network, and Caddy routes the apex domain to
`frontend:4321` and the API domain to `backend:8080`.

Set Cloudflare SSL/TLS encryption mode to **Full (strict)** so Cloudflare also
validates Caddy's origin certificate.

## GitHub Actions

`.github/workflows/infra.yml` runs `terraform plan` on every pull request. On
every push to `main`, it applies Terraform and deploys the Compose stack to the
VPS.

Create these repository or environment variables:

```text
OVH_ENDPOINT
OVH_S3_REGION
OVH_S3_ENDPOINT
TF_STATE_BUCKET
TF_STATE_KEY
CLOUDFLARE_ZONE_ID
OVH_VPS_NAME
OVH_VPS_PLAN_CODE
OVH_VPS_DATACENTER
OVH_VPS_OS
OVH_VPS_IMAGE_ID
VPS_SSH_USER
API_SUBDOMAIN
SITE_DOMAIN
API_DOMAIN
ACME_EMAIL
```

Create these secrets:

```text
OVH_APPLICATION_KEY
OVH_APPLICATION_SECRET
OVH_CONSUMER_KEY
CLOUDFLARE_API_TOKEN
OVH_S3_ACCESS_KEY_ID
OVH_S3_SECRET_ACCESS_KEY
VPS_SSH_PUBLIC_KEY
VPS_SSH_PRIVATE_KEY
BLOG_ENV
BACKEND_ENV
```

`BLOG_ENV` and `BACKEND_ENV` are the complete contents of the respective
`.env` files. The workflow excludes all `.env` files when syncing the
repository, writes those secrets securely on the VPS, and then starts the
stack. The first connection accepts the VPS SSH host key automatically; replace
that with a pinned known-host entry once the VPS is established.

Use `cloudflare_proxied = false` temporarily if you need to diagnose TLS or
origin reachability directly. Keep it `true` for normal Cloudflare proxying.
