terraform {
  required_version = ">= 1.5.0"

  required_providers {
    cloudflare = {
      source  = "cloudflare/cloudflare"
      version = "~> 5.0"
    }
    oci = {
      source  = "oracle/oci"
      version = "~> 8.0"
    }
  }
}

provider "cloudflare" {}

# OCI credentials are supplied by OCI_* environment variables in CI.
provider "oci" {}

data "oci_identity_availability_domains" "available" {
  compartment_id = var.oci_tenancy_ocid
}

locals {
  availability_domain = data.oci_identity_availability_domains.available.availability_domains[var.oci_availability_domain_index].name
}

resource "oci_core_vcn" "site" {
  compartment_id = var.oci_compartment_ocid
  cidr_blocks    = [var.vcn_cidr]
  display_name   = "${var.instance_name}-vcn"
  dns_label      = "sitevcn"
}

resource "oci_core_internet_gateway" "site" {
  compartment_id = var.oci_compartment_ocid
  vcn_id         = oci_core_vcn.site.id
  display_name   = "${var.instance_name}-internet-gateway"
  enabled        = true
}

resource "oci_core_route_table" "public" {
  compartment_id = var.oci_compartment_ocid
  vcn_id         = oci_core_vcn.site.id
  display_name   = "${var.instance_name}-public-routes"

  route_rules {
    destination       = "0.0.0.0/0"
    destination_type  = "CIDR_BLOCK"
    network_entity_id = oci_core_internet_gateway.site.id
  }
}

resource "oci_core_security_list" "public" {
  compartment_id = var.oci_compartment_ocid
  vcn_id         = oci_core_vcn.site.id
  display_name   = "${var.instance_name}-public-security-list"

  dynamic "ingress_security_rules" {
    for_each = toset([22, 80, 443])
    content {
      protocol = "6"
      source   = "0.0.0.0/0"
      tcp_options {
        min = ingress_security_rules.value
        max = ingress_security_rules.value
      }
    }
  }

  egress_security_rules {
    protocol    = "all"
    destination = "0.0.0.0/0"
  }
}

resource "oci_core_subnet" "public" {
  compartment_id             = var.oci_compartment_ocid
  vcn_id                     = oci_core_vcn.site.id
  cidr_block                 = var.public_subnet_cidr
  display_name               = "${var.instance_name}-public-subnet"
  dns_label                  = "public"
  route_table_id             = oci_core_route_table.public.id
  security_list_ids          = [oci_core_security_list.public.id]
  prohibit_public_ip_on_vnic = false
}

resource "oci_core_instance" "site" {
  availability_domain = local.availability_domain
  compartment_id      = var.oci_compartment_ocid
  display_name        = var.instance_name
  shape               = var.oci_instance_shape

  shape_config {
    ocpus         = var.oci_instance_ocpus
    memory_in_gbs = var.oci_instance_memory_gbs
  }

  create_vnic_details {
    subnet_id        = oci_core_subnet.public.id
    assign_public_ip = true
  }

  metadata = {
    ssh_authorized_keys = var.ssh_public_key
  }

  source_details {
    source_type             = "image"
    source_id               = var.oci_image_ocid
    boot_volume_size_in_gbs = var.boot_volume_size_gbs
  }
}

resource "cloudflare_dns_record" "site" {
  zone_id = var.cloudflare_zone_id
  name    = "@"
  type    = "A"
  content = oci_core_instance.site.public_ip
  proxied = var.cloudflare_proxied
  ttl     = 1
  comment = "Managed by Terraform: ${var.instance_name}"
}

resource "cloudflare_dns_record" "api" {
  zone_id = var.cloudflare_zone_id
  name    = var.api_subdomain
  type    = "A"
  content = oci_core_instance.site.public_ip
  proxied = var.cloudflare_proxied
  ttl     = 1
  comment = "Managed by Terraform: ${var.instance_name}"
}
