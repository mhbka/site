terraform {
  required_version = ">= 1.5.0"

  required_providers {
    cloudflare = {
      source  = "cloudflare/cloudflare"
      version = "~> 5.0"
    }
    ovh = {
      source  = "ovh/ovh"
      version = "~> 2.0"
    }
  }
}

provider "cloudflare" {
  # Set CLOUDFLARE_API_TOKEN instead of committing a token here.
}

provider "ovh" {
  # Set OVH_ENDPOINT, OVH_APPLICATION_KEY, OVH_APPLICATION_SECRET, and
  # OVH_CONSUMER_KEY instead of committing credentials here.
}

data "ovh_me" "current" {}

resource "ovh_vps" "site" {
  display_name   = var.vps_name
  image_id       = var.vps_image_id
  ovh_subsidiary = data.ovh_me.current.ovh_subsidiary
  public_ssh_key = var.ssh_public_key

  plan = [{
    duration     = "P1M"
    plan_code    = var.vps_plan_code
    pricing_mode = "default"
    configuration = [
      {
        label = "vps_datacenter"
        value = var.vps_datacenter
      },
      {
        label = "vps_os"
        value = var.vps_os
      },
    ]
  }]
}

# The VPS resource does not expose its assigned IP, so read it once ordering has
# completed. OVH returns the addresses attached to the new VPS here.
data "ovh_vps" "site" {
  service_name = ovh_vps.site.name
}

locals {
  vps_ipv4 = one([
    for ip in data.ovh_vps.site.ips : ip
    if can(regex("^\\d{1,3}(\\.\\d{1,3}){3}$", ip))
  ])
}

resource "cloudflare_dns_record" "site" {
  zone_id = var.cloudflare_zone_id
  name    = "@"
  type    = "A"
  content = local.vps_ipv4
  proxied = var.cloudflare_proxied
  ttl     = 1
  comment = "Managed by Terraform: ${var.vps_name}"
}

resource "cloudflare_dns_record" "api" {
  zone_id = var.cloudflare_zone_id
  name    = var.api_subdomain
  type    = "A"
  content = local.vps_ipv4
  proxied = var.cloudflare_proxied
  ttl     = 1
  comment = "Managed by Terraform: ${var.vps_name}"
}

resource "terraform_data" "bootstrap" {
  triggers_replace = [ovh_vps.site.id]

  connection {
    type        = "ssh"
    host        = local.vps_ipv4
    user        = var.ssh_user
    private_key = file(var.ssh_private_key_path)
  }

  provisioner "remote-exec" {
    inline = [
      "until sudo -n true 2>/dev/null; do sleep 2; done",
      "sudo apt-get update",
      "sudo DEBIAN_FRONTEND=noninteractive apt-get install -y docker.io docker-compose-plugin ufw",
      "sudo systemctl enable --now docker",
      "sudo ufw allow OpenSSH",
      "sudo ufw allow 80/tcp",
      "sudo ufw allow 443/tcp",
      "sudo ufw --force enable",
    ]
  }
}
