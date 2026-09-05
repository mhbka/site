variable "cloudflare_zone_id" {
  description = "Cloudflare zone ID for the site domain."
  type        = string
}

variable "oci_tenancy_ocid" {
  description = "OCI tenancy OCID; also used to query availability domains."
  type        = string
}

variable "oci_compartment_ocid" {
  description = "OCI compartment OCID in which to create the network and instance."
  type        = string
}

variable "instance_name" {
  description = "Display name for the OCI compute instance and related networking."
  type        = string
  default     = "site"
}

variable "oci_availability_domain_index" {
  description = "Zero-based availability-domain position. Index 0 is AP-SINGAPORE-1-AD-1."
  type        = number
  default     = 0
}

variable "oci_instance_shape" {
  description = "OCI compute shape."
  type        = string
  default     = "VM.Standard.A1.Flex"
}

variable "oci_instance_ocpus" {
  description = "OCPUs assigned to the flexible compute shape."
  type        = number
  default     = 1
}

variable "oci_instance_memory_gbs" {
  description = "Memory in GB assigned to the flexible compute shape."
  type        = number
  default     = 6
}

variable "boot_volume_size_gbs" {
  description = "Boot volume size in GB."
  type        = number
  default     = 50
}

variable "vcn_cidr" {
  description = "CIDR range for the VCN."
  type        = string
  default     = "10.0.0.0/16"
}

variable "public_subnet_cidr" {
  description = "CIDR range for the public subnet."
  type        = string
  default     = "10.0.0.0/24"
}

variable "ssh_public_key" {
  description = "SSH public key installed on the Ubuntu instance."
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
