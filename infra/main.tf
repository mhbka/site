terraform {
  required_version = ">= 1.5.0"

  required_providers {
    cloudflare = {
      source  = "cloudflare/cloudflare"
      version = "~> 5.0"
    }
    digitalocean = {
      source  = "digitalocean/digitalocean"
      version = "~> 2.0"
    }
  }
}

provider "cloudflare" {}

# The DigitalOcean provider reads DIGITALOCEAN_TOKEN from the environment.
provider "digitalocean" {}

resource "digitalocean_droplet" "site" {
  name     = var.instance_name
  region   = var.digitalocean_region
  size     = var.digitalocean_droplet_size
  image    = var.digitalocean_image
  ssh_keys = [var.ssh_public_key]

  tags = ["site"]
}

resource "digitalocean_firewall" "site" {
  name        = "${var.instance_name}-firewall"
  droplet_ids = [digitalocean_droplet.site.id]

  dynamic "inbound_rule" {
    for_each = toset([22, 80, 443])
    content {
      protocol         = "tcp"
      port_range       = inbound_rule.value
      source_addresses = ["0.0.0.0/0", "::/0"]
    }
  }

  outbound_rule {
    protocol              = "icmp"
    destination_addresses = ["0.0.0.0/0", "::/0"]
  }

  outbound_rule {
    protocol              = "tcp"
    port_range            = "1-65535"
    destination_addresses = ["0.0.0.0/0", "::/0"]
  }

  outbound_rule {
    protocol              = "udp"
    port_range            = "1-65535"
    destination_addresses = ["0.0.0.0/0", "::/0"]
  }
}

resource "cloudflare_dns_record" "site" {
  zone_id = var.cloudflare_zone_id
  name    = "@"
  type    = "A"
  content = digitalocean_droplet.site.ipv4_address
  proxied = var.cloudflare_proxied
  ttl     = 1
  comment = "Managed by Terraform: ${var.instance_name}"
}

resource "cloudflare_dns_record" "api" {
  zone_id = var.cloudflare_zone_id
  name    = var.api_subdomain
  type    = "A"
  content = digitalocean_droplet.site.ipv4_address
  proxied = var.cloudflare_proxied
  ttl     = 1
  comment = "Managed by Terraform: ${var.instance_name}"
}
