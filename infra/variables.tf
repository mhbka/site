variable "cloudflare_zone_id" {
  description = "Cloudflare zone ID for the site domain."
  type        = string
}

variable "instance_name" {
  description = "Name of the DigitalOcean Droplet."
  type        = string
  default     = "site"
}

variable "digitalocean_region" {
  description = "DigitalOcean region slug for the Droplet."
  type        = string
  default     = "sgp1"
}

variable "digitalocean_droplet_size" {
  description = "DigitalOcean Droplet size slug."
  type        = string
  default     = "s-2vcpu-4gb"
}

variable "digitalocean_image" {
  description = "DigitalOcean image slug for the Droplet."
  type        = string
  default     = "ubuntu-24-04-x64"
}

variable "ssh_public_key" {
  description = "SSH public key installed on the Ubuntu Droplet."
  type        = string
}

variable "api_subdomain" {
  description = "Subdomain that serves the API."
  type        = string
  default     = "api"
}

variable "cloudflare_proxied" {
  description = "Whether Cloudflare proxies the DNS records."
  type        = bool
  default     = true
}
