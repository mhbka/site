# Infrastructure

This configuration creates an Oracle Cloud Infrastructure (OCI) Ubuntu 24.04
instance, its VCN/public subnet and firewall rules, and Cloudflare A records
for the zone apex and `api` subdomain. The GitHub Actions workflow installs
Docker and starts the Compose application and Caddy.

## OCI access and permissions

Create an OCI API signing key for a user that can manage Compute and Virtual
Network resources in the target compartment. The workflow uses API-key
authentication; it does not require a manually-created instance, VCN, or
subnet. OCI documents the required tenancy OCID, user OCID, fingerprint,
private key, and region for this authentication method.

Terraform selects the first availability domain in Singapore, which corresponds
to `AP-SINGAPORE-1-AD-1`. It selects the latest matching Canonical Ubuntu 24.04
image for `VM.Standard.A1.Flex`.

The public OCI security list and UFW both allow TCP 22, 80, and 443. Narrow SSH
to a fixed source range after establishing a suitable access path.

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
OCI_REGION                         # ap-singapore-1
OCI_TENANCY_OCID
OCI_USER_OCID
OCI_COMPARTMENT_OCID                # Optional; defaults to the root tenancy compartment
OCI_INSTANCE_NAME                  # e.g. site
OCI_AVAILABILITY_DOMAIN_INDEX      # 0 for AP-SINGAPORE-1-AD-1
OCI_INSTANCE_OCPUS                 # 1 initially
OCI_INSTANCE_MEMORY_GBS            # 6 initially
CLOUDFLARE_ZONE_ID
S3_REGION                          # auto for Cloudflare R2
S3_ENDPOINT                        # R2 account endpoint
TF_STATE_BUCKET
TF_STATE_KEY
OCI_SSH_USER                       # ubuntu
API_SUBDOMAIN                      # api
SITE_DOMAIN
API_DOMAIN
ACME_EMAIL
```

Create these GitHub Actions secrets:

```text
OCI_PRIVATE_KEY                    # PEM private half of the OCI API signing key
OCI_FINGERPRINT                    # Fingerprint of that OCI API signing key
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
