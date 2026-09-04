terraform {
  # Bucket and key are supplied by GitHub Actions during `terraform init`.
  backend "s3" {
    use_lockfile                = true
    skip_credentials_validation = true
    skip_region_validation      = true
    skip_requesting_account_id  = true
    skip_s3_checksum            = true
  }
}
