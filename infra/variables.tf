variable "cloudflare_zone_id" {
  description = "Cloudflare zone ID for the site domain."
  type        = string
}

variable "vps_name" {
  description = "Display name for the OVH VPS."
  type        = string
  default     = "site"
}

variable "vps_plan_code" {
  description = "OVH VPS product plan code, such as vps-le-2-2-40."
  type        = string
}

variable "vps_datacenter" {
  description = "OVH VPS datacenter code available for the chosen plan."
  type        = string
}

variable "vps_os" {
  description = "Operating-system value accepted by the selected OVH VPS plan."
  type        = string
  default     = "Debian 12"
}

variable "vps_image_id" {
  description = "OVH image ID used to install the supplied SSH public key."
  type        = string
}

variable "ssh_public_key" {
  description = "SSH public key to install on the VPS."
  type        = string
}

variable "ssh_private_key_path" {
  description = "Local path to the matching private SSH key; used only for bootstrap."
  type        = string
  sensitive   = true
}

variable "ssh_user" {
  description = "SSH user created by the selected image. Debian images commonly use debian."
  type        = string
  default     = "debian"
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
