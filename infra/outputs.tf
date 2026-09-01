output "vps_ipv4" {
  description = "Public IPv4 address assigned to the VPS."
  value       = local.vps_ipv4
}

output "vps_service_name" {
  description = "OVH VPS service name."
  value       = ovh_vps.site.name
}
